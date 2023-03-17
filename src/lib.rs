use nix::unistd::Uid;
use std::path::Path;

pub struct Config<'conf> {
    pub kernel_boot_path: &'conf Path,
    pub initramfs_boot_files: Vec<&'conf Path>,
}

#[derive(Debug)]
pub enum BuilderErr {
    NoPrivileges,
}

pub struct KernelBuilder<'conf> {
    config: Config<'conf>,
    versions: Vec<String>,
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
        if !Uid::effective().is_root() {
            return Err(BuilderErr::NoPrivileges);
        }

        Ok(())
    }

    fn get_available_version(&mut self) {
        if self.versions.is_empty() {
            self.versions = std::fs::read_dir(Self::LINUX_PATH)
                .unwrap()
                .map(|direntry| direntry.unwrap().path())
                .filter(|p| p.is_dir())
                .map(|p| p.to_str().unwrap().to_owned())
                .collect::<Vec<_>>();
        }
    }

    pub fn start_build_process(&self) {
        self.prompt_for_kernel_version();
        self.prompt_for_modules_install();
        self.prompt_for_initramfs_gen();
        // TODO:
        // build kernel and copy to boot directory
        // build and install modules
        // build initramfs and change loader entries
        todo!()
    }

    fn prompt_for_kernel_version(&self) {
        todo!()
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
