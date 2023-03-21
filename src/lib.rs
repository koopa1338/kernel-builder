use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use dialoguer::{console::Term, Confirm};
use indicatif::ProgressBar;
#[cfg(not(debug_assertions))]
use nix::unistd::Uid;
use std::num::NonZeroUsize;
use std::{
    os::unix,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

mod error;
pub use error::BuilderErr;

pub struct Config<'conf> {
    /// Path to the kernel bz image on the boot partition
    pub kernel_file_path: &'conf Path,
    /// List of files where the initramfs version has to be updated, e.g. boot entries config files
    pub boot_entry_config: &'conf Path,
    /// path to the `.config` file that will be symlinked
    pub kernel_config_file_path: &'conf Path,
}

#[derive(Clone, Debug)]
struct VersionEntry {
    path: PathBuf,
    version_string: String,
}

pub struct KernelBuilder<'conf> {
    config: Config<'conf>,
    versions: Vec<VersionEntry>,
}

impl<'conf> KernelBuilder<'conf> {
    const LINUX_PATH: &str = "/usr/src";

    pub fn new(config: Config<'conf>) -> Self {
        let mut builder = Self {
            config,
            versions: vec![],
        };
        builder.get_available_version();

        builder
    }

    pub fn check_privileges(&self) -> Result<(), BuilderErr> {
        #[cfg(not(debug_assertions))]
        if !Uid::effective().is_root() {
            return Err(BuilderErr::NoPrivileges);
        }

        Ok(())
    }

    fn get_available_version(&mut self) {
        if self.versions.is_empty() {
            if let Ok(directories) = std::fs::read_dir(Self::LINUX_PATH) {
                self.versions = directories
                    .filter_map(|direntry| direntry.ok())
                    .map(|dir| dir.path())
                    .filter(|path| path.starts_with(Self::LINUX_PATH) && !path.is_symlink())
                    .filter_map(|path| {
                        path.strip_prefix(Self::LINUX_PATH).ok().and_then(|p| {
                            let tmp = p.to_owned();
                            let version_string = tmp.to_string_lossy();
                            (version_string.starts_with("linux")
                                && version_string.ends_with("gentoo"))
                            .then_some(VersionEntry {
                                path: path.to_owned(),
                                version_string: version_string.to_string(),
                            })
                        })
                    })
                    .collect::<Vec<_>>();
            }
        }
    }

    pub fn build(&self) -> Result<(), BuilderErr> {
        let version_entry = self.prompt_for_kernel_version();

        // create symlink from /usr/src/.config
        let link = version_entry.path.join(".config");
        if !link.exists() {
            let dot_config = PathBuf::from(Self::LINUX_PATH).join(".config");
            if !dot_config.exists() || !dot_config.is_file() {
                return Err(BuilderErr::KernelConfigMissing);
            }

            unix::fs::symlink(dot_config, link)
                .map_err(|_| BuilderErr::LinkingFileError("failed to create symlink".into()))?;
        }

        let linux = PathBuf::from(Self::LINUX_PATH).join("linux");
        let linux_target = linux.read_link().map_err(|_| {
            BuilderErr::LinkingFileError(format!("failed to read symlink for {linux:?}"))
        })?;

        if linux_target.to_string_lossy() != version_entry.version_string {
            std::fs::remove_file(&linux).map_err(|_| {
                BuilderErr::LinkingFileError("failed to delete linux symlink".into())
            })?;
            unix::fs::symlink(&version_entry.path, linux)
                .map_err(|_| BuilderErr::LinkingFileError("failed to create symlink".into()))?;
        }

        self.build_kernel(&version_entry)?;

        if self.confirm_prompt("Do you want to install kernel modules?")? {
            self.install_kernel_modules(&version_entry)?;
        }

        if self.confirm_prompt("Do you want to generate initramfs with dracut?")? {
            self.generate_initramfs(&version_entry)?;
        }

        Ok(())
    }

    fn build_kernel(&self, version_entry: &VersionEntry) -> Result<(), BuilderErr> {
        let threads: NonZeroUsize =
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message("Compiling kernel...");
        Command::new("make")
            .current_dir(&version_entry.path)
            .arg("-j")
            .arg(threads.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                BuilderErr::KernelBuildFail(format!("failed to spawn build process: {err}"))
            })?
            .wait()
            .map_err(|err| BuilderErr::KernelBuildFail(format!("failed to build kernel: {err}")))?;
        pb.finish_with_message("Finished compiling Kernel");
        std::fs::copy(
            &version_entry.path.join("arch/x86/boot/bzImage"),
            self.config.kernel_file_path,
        )
        .map_err(|_| {
            BuilderErr::KernelBuildFail(format!(
                "failed to copy kernel to {:?}",
                self.config.kernel_file_path
            ))
        })?;

        Ok(())
    }

    fn install_kernel_modules(&self, version_entry: &VersionEntry) -> Result<(), BuilderErr> {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message("Install kernel modules");
        Command::new("make")
            .current_dir(&version_entry.path)
            .arg("modules_install")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                BuilderErr::KernelBuildFail(format!(
                    "failed to spawn kernel module install process: {err}"
                ))
            })?
            .wait()
            .map_err(|err| {
                BuilderErr::KernelBuildFail(format!("failed to install kernel modules: {err}"))
            })?;
        pb.finish_with_message("Finished installing modules");

        Ok(())
    }

    fn generate_initramfs(&self, version_entry: &VersionEntry) -> Result<(), BuilderErr> {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message("Gen initramfs");
        Command::new("dracut")
            .current_dir(&version_entry.path)
            .args(&[
                "--hostonly",
                "--kver",
                version_entry
                    .version_string
                    .strip_prefix("linux-")
                    .unwrap_or("uknown-gentoo"),
                "--force",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                BuilderErr::KernelBuildFail(format!(
                    "failed to spawn process to generate initramfs: {err}"
                ))
            })?
            .wait()
            .map_err(|err| {
                BuilderErr::KernelBuildFail(format!("failed to generate initramfs: {err}"))
            })?;
        pb.finish_with_message("Finished initramfs");
        // TODO: replace initramfs version in loader.conf

        Ok(())
    }

    fn prompt_for_kernel_version(&self) -> VersionEntry {
        let versions = self
            .versions
            .clone()
            .into_iter()
            .map(|v| v.version_string)
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick version to build and install")
            .items(versions.as_slice())
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap()
            .unwrap();
        self.versions[selection].clone()
    }

    fn confirm_prompt(&self, message: &str) -> Result<bool, BuilderErr> {
        Confirm::new()
            .with_prompt(message)
            .interact()
            .map_err(|_| BuilderErr::PromptError)
    }
}
