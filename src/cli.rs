#[derive(Debug)]
pub struct SkipArgs {
    pub no_build: bool,
    #[cfg(feature = "dracut")]
    pub no_initramfs: bool,
    pub no_modules: bool,
}

impl SkipArgs {
    const HELP: &str = r#"
Kernel Builder
USAGE:
  kernel-builder [OPTIONS]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --no-build          skip build
  --no-initramfs      skip generating initramfs (only if compiled with dracut feature)
  --no-modules        skip installing kernel modules
"#;

    pub fn parse_args() -> Self {
        let mut pargs = pico_args::Arguments::from_env();

        // Help has a higher priority and should be handled separately.
        if pargs.contains(["-h", "--help"]) {
            print!("{}", Self::HELP);
            std::process::exit(0);
        }

        let args = Self {
            no_build: pargs.contains("--no-build"),
            #[cfg(feature = "dracut")]
            no_initramfs: pargs.contains("--no-initramfs"),
            no_modules: pargs.contains("--no-modules"),
        };

        args
    }
}
