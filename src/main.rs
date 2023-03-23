use gentoo_kernel_builder::{BuilderErr, Config, KernelBuilder};
use std::path::Path;

fn main() -> Result<(), BuilderErr> {
    let kernel_config_file_path = Path::new(KernelBuilder::LINUX_PATH).join(".config");
    let config = Config {
        kernel_file_path: Path::new("/boot/vmlinuz-linux-gentoo"),
        initramfs_file_path: Path::new("/boot/initramfs-linux-gentoo"),
        kernel_config_file_path: kernel_config_file_path.as_path(),
    };
    let kernel_builder = KernelBuilder::new(config);
    kernel_builder.check_privileges()?;
    kernel_builder.build()?;

    Ok(())
}
