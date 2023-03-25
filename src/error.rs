use config::ConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuilderErr {
    #[error("Error retrieving root privileges")]
    NoPrivileges,
    #[error("Missing kernel configuration file")]
    KernelConfigMissing,
    #[error("Missing option in kernel configuration file: {0}")]
    KernelConfigMissingOption(String),
    #[error("Error building kernel: {0}")]
    KernelBuildFail(std::io::Error),
    #[error("Symlinking file failed: {0}")]
    LinkingFileError(std::io::Error),
    #[error("Could not parse config file: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("Could not create prompt: {0}")]
    PromptError(std::io::Error),
}
