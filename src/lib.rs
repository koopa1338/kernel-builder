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
    versions: Vec<String>
}

impl<'conf> KernelBuilder<'conf> {
    pub fn new(config: Config<'conf>) -> Self {
        Self {
            config,
            versions: vec![],
        }
    }

    pub fn check_privileges(&self) -> Result<(), BuilderErr> {
        if !Uid::effective().is_root() {
            return Err(BuilderErr::NoPrivileges);
        }

        Ok(())
    }

    pub fn get_available_version(&mut self) -> &Vec<String> {
        if self.versions.is_empty() {
            let versions = vec![];
            // TODO
            self.versions = versions;
        }

        &self.versions
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
