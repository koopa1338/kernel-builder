# Kernel Builder

The Kernel Builder is a application written in rust that scans for available
kernel sources in a configured directory and allows you to select which kernel
to build, as well as install the modules, copy the compiled kernel to the boot
partition, as well as generating the necessary initramfs.

## Prerequisites

- Basic knowledge of kernel configuration and compilation
- to use the initramfs generation you have to enable the `dracut` feature and
  dracut has to be installed on your system

## Installation

```sh
git clone https://github.com/koopa1338/kernel-builder
cd kernel-builder
cargo install --path .
```

or install it with cargo install from crates.io
```sh
cargo install kernel-builder
```

You also need a `config.toml` with the needed paths configured in `$HOME/.config/kb/config.toml`:
```toml
kernel = "/boot/vmlinuz-linux"
initramfs = "/boot/initramfs-linux" # Optional, only needed if `dracut` feature is enabled
kernel-config = "/usr/src/.config"
kernel-src
```

## Usage

If correctly setup you should just run `kernel-builder`, it should ask
for root permission if not alread run as root. You can override options by
setting environment variables prefixed with `KB_`. For example to override the
kernel path:

```sh
KB_KERNEL=/boot/efi/vmlinuz-linux-lts kernel-builder
```

## TODO

- [ ] support command line options to skip prompts
- [ ] support bootloader update (e.g. `update-grub`)
- [ ] support `genkernel` as beside `dracut` for initramfs
- [ ] before copying to boot folder, backup old kernel and ramfs to fallback version

 
## Contributing

There is still room for improvements, so if you would like to contribute to the
project, please feel free to submit a pull request or open an issue.

# License

This script is released under the EUPL License. See the LICENSE file for more
information.
