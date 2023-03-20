use gentoo_kernel_builder::{BuilderErr, Config, KernelBuilder};
use std::path::Path;

fn main() -> Result<(), BuilderErr> {
    let config = Config {
        kernel_boot_path: Path::new("/boot"),
        initramfs_boot_files: vec![],
    };
    let kernel_builder = KernelBuilder::new(config);
    kernel_builder.check_privileges()?;
    kernel_builder.start_build_process();
    /*
        3. get prompt for selection
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
