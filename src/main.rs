use config::{Config, Environment, File};
use kernel_builder::{BuilderErr, KBConfig, KernelBuilder};
use std::path::PathBuf;

use kernel_builder::Args;

fn main() -> Result<(), BuilderErr> {
    let mut settings_path = if let Ok(xdg_env) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_env)
    } else {
        PathBuf::from(std::env!("HOME")).join(".config")
    };
    settings_path.push("kernel-builder/config");
    let settings = Config::builder()
        .add_source(File::with_name(settings_path.to_string_lossy().as_ref()).required(true))
        .add_source(Environment::with_prefix("KB"))
        .build()?;

    let config = settings.try_deserialize::<KBConfig>()?;
    let kernel_builder = KernelBuilder::new(config);

    let cli_args = Args::parse_args();
    sudo::escalate_if_needed().map_err(|_| BuilderErr::NoPrivileges)?;
    kernel_builder.build(&cli_args)?;

    Ok(())
}
