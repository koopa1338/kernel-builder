#[derive(Debug)]
pub struct SkipArgs {
    pub skip_build: bool,
    #[cfg(feature = "dracut")]
    pub skip_initramfs: bool,
    pub skip_modules: bool,
}

impl SkipArgs {
    const HELP: &str = r#"
Kernel Builder
USAGE:
  kernel-builder [OPTIONS]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --skip-build          skip build
  --skip-initramfs      skip generating initramfs (only with dracut feature)
  --skip-modules        skip installing kernel modules
"#;

    pub fn parse_args() -> Self {
        let mut pargs = pico_args::Arguments::from_env();

        // Help has a higher priority and should be handled separately.
        if pargs.contains(["-h", "--help"]) {
            print!("{}", Self::HELP);
            std::process::exit(0);
        }

        let args = Self {
            skip_build: pargs.contains("--skip-build"),
            #[cfg(feature = "dracut")]
            skip_initramfs: pargs.contains("--skip-initramfs"),
            skip_modules: pargs.contains("--skip-modules"),
        };

        args
    }
}
