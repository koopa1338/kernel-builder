use gentoo_kernel_builder::{BuilderErr, Config, KernelBuilder};
use std::path::Path;

fn main() -> Result<(), BuilderErr> {
    let kernel_config_file_path = Path::new(KernelBuilder::LINUX_PATH).join(".config");
    let config = Config {
        kernel_file_path: Path::new("/boot/vmlinuz-linux-gentoo"),
        initramfs_file_path: Path::new("/boot/initramfs-linux-gentoo"),
        boot_entry_config: Path::new("/boot/loader/entries/gentoo.conf"),
        kernel_config_file_path: kernel_config_file_path.as_path(),
    };
    let kernel_builder = KernelBuilder::new(config);
    kernel_builder.check_privileges()?;
    kernel_builder.build()?;
    /*
        4. change the symlink in /usr/src for linux to the new kernel version
            - eselect picks this up correctly if you run `eselect kernel list`
        5. link config to selected kernel directory if not already there
            - `ln -sf /usr/src/.config /usr/src/linux/.config`
        6. build the kernel
            - `make -j7`
        7. install modules
            - `make install_modules`
        8. copy kernel to `/boot/vmlinuz-linux-gentoo (maybe config or env)
        9. build initramfs with dracut
            - `dracut --hostonly --kver <version>-gentoo --force`
        10. change the initramfs version in loader entries (path as config or env)
    */

    Ok(())
}
