#[derive(Debug)]
pub enum BuilderErr {
    NoPrivileges,
    KernelConfigMissing,
    KernelBuildFail(String),
    LinkingFileError(String),
    ConfigParseError(String),
    ConfigFileMissing,
    PromptError,
}

impl std::fmt::Display for BuilderErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NoPrivileges => "builder has to be startet as root",
            Self::KernelConfigMissing => "Missing .config file in /usr/src",
            Self::LinkingFileError(msg)
            | Self::KernelBuildFail(msg)
            | Self::ConfigParseError(msg) => msg,
            Self::PromptError => "error setting up prompt for user input",
            Self::ConfigFileMissing => "Config missing, put a config file in $HOME/.config/gkb/",
        };
        write!(f, "BuildErr: {message}")
    }
}

impl std::error::Error for BuilderErr {}
