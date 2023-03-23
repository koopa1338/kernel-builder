# Gentoo Kernel Builder

The Gentoo Kernel Builder is a application written in rust that scans for
available kernel sources on your gentoo system and allows you to select which
kernel to build, as well as install the modules, copy the compiled kernel to
the boot partition, as well as generating the necessary initramfs.

## Prerequisites

- A working installation of Gentoo Linux
- Basic knowledge of kernel configuration and compilation

## Installation

```sh
git clone https://github.com/koopa1338/gentoo-kernel-builder
cd gentoo-kernel-builder
cargo install --path .
```

You also need a `config.toml` with the needed paths configured in `$HOME/.config/gkb/config.toml`:
```toml
kernel = "/boot/vmlinuz-linux-gentoo"
initramfs = "/boot/initramfs-linux-gentoo"
kernel-config = "/usr/src/.config"
```

## Usage

If correctly setup you should just run `gentoo-kernel-builder`, it should ask
for root permission if not alread run as root. You can override options by
setting environment variables prefixed with `GKB_`. For example to override the
kernel path:

```sh
GKB_KERNEL=/boot/efi/vmlinuz-linux-gentoo gentoo-kernel-builder
```

## Contributing

This project is in early development and is not yet usable. However,
contributions are welcome! If you would like to contribute to the project,
please feel free to submit a pull request or open an issue.

# License

This script is released under the EUPL License. See the LICENSE file for more
information.
