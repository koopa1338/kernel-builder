use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use dialoguer::{console::Term, Confirm};
use indicatif::ProgressBar;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::num::NonZeroUsize;
use std::{
    os::unix,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

mod error;
pub use error::BuilderErr;
mod cli;
pub use cli::Args;

#[derive(Debug, Deserialize)]
pub struct KBConfig {
    /// Path to the kernel bz image on the boot partition
    #[serde(rename = "kernel")]
    pub kernel_file_path: PathBuf,
    /// Path to the initramfs on the boot partition
    #[serde(rename = "initramfs")]
    pub initramfs_file_path: Option<PathBuf>,
    /// path to the `.config` file that will be symlinked
    #[serde(rename = "kernel-config")]
    pub kernel_config_file_path: PathBuf,
    /// path to the kernel sources
    #[serde(rename = "kernel-src")]
    pub kernel_src: PathBuf,
}

#[derive(Clone, Debug)]
struct VersionEntry {
    path: PathBuf,
    version_string: String,
}

#[derive(Debug)]
pub struct KernelBuilder {
    config: KBConfig,
    versions: Vec<VersionEntry>,
}

impl KernelBuilder {
    pub const LINUX_PATH: &str = "/usr/src";

    #[must_use]
    pub fn new(config: KBConfig) -> Self {
        let mut builder = Self {
            config,
            versions: vec![],
        };
        builder.get_available_version();

        builder
    }

    fn get_available_version(&mut self) {
        if self.versions.is_empty() {
            if let Ok(directories) = std::fs::read_dir(&self.config.kernel_src) {
                self.versions = directories
                    .filter_map(|dir| dir.ok().map(|d| d.path()))
                    .filter(|path| path.starts_with(&self.config.kernel_src) && !path.is_symlink())
                    .filter_map(|path| {
                        path.strip_prefix(&self.config.kernel_src)
                            .ok()
                            .and_then(|p| {
                                let tmp = p.to_owned();
                                let version_string = tmp.to_string_lossy();
                                version_string
                                    .starts_with("linux-")
                                    .then_some(VersionEntry {
                                        path: path.clone(),
                                        version_string: version_string.to_string(),
                                    })
                            })
                    })
                    .collect::<Vec<_>>();
            }
        }
    }

    ///
    /// # Errors
    ///
    /// - Error on missing kernel config
    /// - Failing creating symlinks
    /// - Failing kernel build
    ///
    /// if selected:
    /// - Failing installing kernel modules
    /// - Failing generating initramfs
    pub fn build(&self, cli: &Args) -> Result<(), BuilderErr> {
        let version_entry = self.prompt_for_kernel_version();
        let VersionEntry {
            path,
            version_string,
        } = &version_entry;

        // create symlink from /usr/src/.config
        let link = path.join(".config");
        if !link.exists() {
            let dot_config = &self.config.kernel_config_file_path;
            if !dot_config.exists() || !dot_config.is_file() {
                return Err(BuilderErr::KernelConfigMissing);
            }

            unix::fs::symlink(dot_config, link).map_err(BuilderErr::LinkingFileError)?;
        }

        let linux = PathBuf::from(&self.config.kernel_src).join("linux");
        let linux_target = linux.read_link().map_err(BuilderErr::LinkingFileError)?;

        if linux_target.to_string_lossy() != *version_string {
            std::fs::remove_file(&linux).map_err(BuilderErr::LinkingFileError)?;
            unix::fs::symlink(path, linux).map_err(BuilderErr::LinkingFileError)?;
        }

        if cli.menuconfig {
            Self::make_menuconfig(path)?;
        }

        if !cli.no_build {
            self.build_kernel(path)?;
        }

        if !cli.no_modules && Self::confirm_prompt("Do you want to install kernel modules?")? {
            Self::install_kernel_modules(path)?;
        }

        #[cfg(feature = "dracut")]
        if !cli.no_initramfs
            && Self::confirm_prompt("Do you want to generate initramfs with dracut?")?
        {
            self.generate_initramfs(&version_entry)?;
        }
        Ok(())
    }

    fn build_kernel(&self, path: &Path) -> Result<(), BuilderErr> {
        let threads: NonZeroUsize =
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));

        let mut cmd = Command::new("make")
            .current_dir(path)
            .args(["-j", &threads.to_string()])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(BuilderErr::KernelBuildFail)?;

        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines {
                pb.set_message(format!(
                    "Compiling kernel: {}",
                    line.map_err(BuilderErr::KernelBuildFail)?
                ));
            }
        }

        cmd.wait().map_err(BuilderErr::KernelBuildFail)?;

        pb.finish_with_message("Finished compiling Kernel");
        std::fs::copy(
            path.join("arch/x86/boot/bzImage"),
            self.config.kernel_file_path.clone(),
        )
        .map_err(BuilderErr::KernelBuildFail)?;

        Ok(())
    }

    fn make_menuconfig(path: &Path) -> Result<(), BuilderErr> {
        let mut cmd = Command::new("make")
            .current_dir(path)
            .arg("menuconfig")
            .spawn()
            .map_err(|_| BuilderErr::MenuconfigError)?;

        cmd.wait().map_err(|_| BuilderErr::MenuconfigError)?;

        Ok(())
    }

    fn install_kernel_modules(path: &Path) -> Result<(), BuilderErr> {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message("Install kernel modules");
        Command::new("make")
            .current_dir(path)
            .arg("modules_install")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(BuilderErr::KernelBuildFail)?
            .wait()
            .map_err(BuilderErr::KernelBuildFail)?;
        pb.finish_with_message("Finished installing modules");

        Ok(())
    }

    #[cfg(feature = "dracut")]
    fn generate_initramfs(
        &self,
        VersionEntry {
            path,
            version_string,
        }: &VersionEntry,
    ) -> Result<(), BuilderErr> {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        let initramfs_file_path = &self
            .config
            .initramfs_file_path
            .clone()
            .ok_or(BuilderErr::KernelConfigMissingOption("initramfs".into()))?;
        let mut cmd = Command::new("dracut")
            .current_dir(path)
            .args([
                "--hostonly",
                "--kver",
                version_string.strip_prefix("linux-").unwrap(),
                "--force",
                initramfs_file_path.to_string_lossy().as_ref(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(BuilderErr::KernelBuildFail)?;

        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines {
                pb.set_message(format!(
                    "Generating initramfs: {}",
                    line.map_err(BuilderErr::KernelBuildFail)?
                ));
            }
        }

        cmd.wait().map_err(BuilderErr::KernelBuildFail)?;
        pb.finish_with_message("Finished initramfs");

        Ok(())
    }

    fn prompt_for_kernel_version(&self) -> VersionEntry {
        let versions = self
            .versions
            .clone()
            .into_iter()
            .map(|v| v.version_string)
            .rev() // display the newest version at the top
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick version to build and install")
            .items(versions.as_slice())
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap()
            .unwrap();
        // we display the versions in reverse so the index has to be calculated...
        let index = (self.versions.len() - selection)
            .checked_sub(1)
            .unwrap_or(0);
        self.versions[index].clone()
    }

    fn confirm_prompt(message: &str) -> Result<bool, BuilderErr> {
        Confirm::new()
            .with_prompt(message)
            .interact()
            .map_err(BuilderErr::PromptError)
    }
}
