# dipc

[![crates.io](https://img.shields.io/crates/v/dipc)](https://crates.io/crates/dipc)

doprz' image palette converter

Convert your favorite images and wallpapers with your favorite color palettes/themes

## Color Palettes/Themes

- catppuccin
- gruvbox
- gruvbox-material
- nord
- rose-pine

## Examples

| Original | Converted |
| -------- | --------- |
| ![Magic view of the Gosausee in the Austrian Alps](images/ivan-rohovchenko-pkLBb75JnHc-unsplash.jpg) | ![Magic view of the Gosausee in the Austrian Alps_nord](images/ivan-rohovchenko-pkLBb75JnHc-unsplash_nord-Polar%20Night-Snow%20Storm-Frost.png) |
| ![Human vs Nature](images/victor-rosario-pa9sROVpkgQ-unsplash.jpg) | ![Human vs Nature_gruvbox](images/victor-rosario-pa9sROVpkgQ-unsplash_gruvbox-Dark%20mode-Light%20mode.png) |

### Image Credits

Victor Rosario - [https://unsplash.com/@vrrosario](https://unsplash.com/@vrrosario)

Ivan Rohovchenko - [https://unsplash.com/@ivrn](https://unsplash.com/@ivrn)

## Installation

### Cargo

```sh
cargo install dipc
```

### From Source

To build and install from source, first checkout the tag or branch you want to install, then run
```sh
cargo install --path .
```
This will build and install `dipc` in your `~/.cargo/bin`. Make sure that `~/.cargo/bin` is in your `$PATH` variable.

## Usage

```
Convert your favorite images and wallpapers with your favorite color palettes/themes

Usage: dipc [OPTIONS] --image <FILE> --color-palette <COLOR_PALETTE> <--all|--color-palette-variation <COLOR_PALETTE_VARIATION>>

Options:
  -i, --image <FILE>
          The image to process
      --color-palette <COLOR_PALETTE>
          The color palette to use [possible values: catppuccin, gruvbox, gruvbox-material, nord, rose-pine]
  -a, --all
          Use all color palette variations
      --color-palette-variation <COLOR_PALETTE_VARIATION>
          Color palette variation(s) to use
  -v, --verbose...
          Verbose mode (-v, -vv, -vvv)
  -h, --help
          Print help
  -V, --version
          Print version
```

## License

`rs-cube` is dual-licensed under the terms of both the MIT License and the Apache License 2.0

SPDX-License-Identifier: MIT OR Apache-2.0
