#[derive(Debug)]
pub struct Args {
    pub no_build: bool,
    #[cfg(feature = "dracut")]
    pub no_initramfs: bool,
    pub no_modules: bool,
    pub menuconfig: bool,
    pub replace: bool,
}

impl Args {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const HELP: &'static str = r#"
Kernel Builder
USAGE:
  kernel-builder [OPTIONS]
FLAGS:
  -h, --help            Prints help information
  -v, --version         Print version
OPTIONS:
  --no-build          skip build
  --no-initramfs      skip generating initramfs (only if compiled with dracut feature)
  --no-modules        skip installing kernel modules
  --menuconfig        open menuconfig for kernel configuration
  --replace           replace the current installed kerne (useful if you have configured to keep the last kernel)
"#;

    #[must_use]
    pub fn parse_args() -> Self {
        let mut pargs = pico_args::Arguments::from_env();

        // Help has a higher priority and should be handled separately.
        if pargs.contains(["-h", "--help"]) {
            print!("{}", Self::HELP);
            std::process::exit(0);
        }

        if pargs.contains(["-v", "--version"]) {
            println!("kernel-builder v{}", Self::VERSION);
            std::process::exit(0);
        }

        Self {
            no_build: pargs.contains("--no-build"),
            #[cfg(feature = "dracut")]
            no_initramfs: pargs.contains("--no-initramfs"),
            no_modules: pargs.contains("--no-modules"),
            menuconfig: pargs.contains("--menuconfig"),
            replace: pargs.contains("--replace"),
        }
    }
}
