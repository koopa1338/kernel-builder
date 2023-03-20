use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
#[cfg(not(debug_assertions))]
use nix::unistd::Uid;
use std::path::{Path, PathBuf};

pub struct Config<'conf> {
    pub kernel_boot_path: &'conf Path,
    pub initramfs_boot_files: Vec<&'conf Path>,
}

#[derive(Debug)]
pub enum BuilderErr {
    NoPrivileges,
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
                                path: p.to_owned(),
                                version_string: version_string.to_string(),
                            })
                        })
                    })
                    .collect::<Vec<_>>();
            }
        }
    }

    pub fn start_build_process(&self) {
        let version_entry = self.prompt_for_kernel_version();
        // self.prompt_for_modules_install();
        // self.prompt_for_initramfs_gen();
        // TODO:
        // build kernel and copy to boot directory
        // build and install modules
        // build initramfs and change loader entries
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

    fn prompt_for_modules_install(&self) {
        todo!()
    }

    fn prompt_for_initramfs_gen(&self) {
        todo!()
    }
}

impl std::fmt::Display for BuilderErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            BuilderErr::NoPrivileges => "builder has to be startet as root",
        };
        write!(f, "NoPriviligesError: {}", message)
    }
}

impl std::error::Error for BuilderErr {}
