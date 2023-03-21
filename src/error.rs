#[derive(Debug)]
pub enum BuilderErr {
    NoPrivileges,
    KernelConfigMissing,
    KernelBuildFail(String),
    LinkingFileError(String),
    PromptError,
}

impl std::fmt::Display for BuilderErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            BuilderErr::NoPrivileges => "builder has to be startet as root",
            BuilderErr::KernelConfigMissing => "Missing .config file in /usr/src",
            BuilderErr::LinkingFileError(msg) => msg,
            BuilderErr::KernelBuildFail(msg) => msg,
            BuilderErr::PromptError => "error setting up prompt for user input",
        };
        write!(f, "BuildErr: {}", message)
    }
}

impl std::error::Error for BuilderErr {}

