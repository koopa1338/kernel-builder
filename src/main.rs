use config::{Config, Environment, File};
use kernel_builder::{BuilderErr, KBConfig, KernelBuilder};
use std::path::PathBuf;
use sudo;

fn main() -> Result<(), BuilderErr> {
    let settings_path = if let Ok(xdg_env) = std::env::var("XDG_CONFIG_HOME") {
        let mut xdg = PathBuf::from(xdg_env);
        xdg.push("kernel-builder/config");
        xdg
    } else {
        let mut home = PathBuf::from(std::env!("HOME"));
        home.push(".config/kernel-builder/config");
        home
    };

    let settings = Config::builder()
        .add_source(File::with_name(settings_path.to_string_lossy().as_ref()).required(true))
        .add_source(Environment::with_prefix("KB"))
        .build()?;

    let config = settings.try_deserialize::<KBConfig>()?;
    let kernel_builder = KernelBuilder::new(config);

    sudo::escalate_if_needed().map_err(|_| BuilderErr::NoPrivileges)?;
    kernel_builder.build()?;

    Ok(())
}
