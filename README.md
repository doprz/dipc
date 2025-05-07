# dipc

[![crates.io](https://img.shields.io/crates/v/dipc)](https://crates.io/crates/dipc)
[![Homebrew](https://img.shields.io/homebrew/v/dipc)](https://formulae.brew.sh/formula/dipc)

<picture>
    <source media="(prefers-color-scheme: dark)" srcset="images/dipc_dark.png">
    <img alt="dipc light icon" src="images/dipc_light.png">
</picture>

doprz' image palette converter

Convert your favorite images and wallpapers with your favorite color palettes/themes

## Color Palettes/Themes

- catppuccin
- dracula
- edge
- everforest
- gruvbox
- gruvbox-material
- nord
- onedark
- rose-pine
- solarized
- tokyo-night

## Examples

![dipc examples](images/dipc_examples.png)

### Image Credits

Paul Bill - [https://unsplash.com/@hoffman11](https://unsplash.com/@hoffman11)

Adrien Vajas - [https://unsplash.com/@adrien_vj](https://unsplash.com/@adrien_vj)

Filipp Romanovski - [https://unsplash.com/@filipp_roman_photography](https://unsplash.com/@filipp_roman_photography)

## Installation

### Homebrew

```sh
brew install dipc
```

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

### Docker
## Installation

```
user@machine$: git clone https://github.com/doprz/dipc
user@machine$: cd dipc
user@machine$: docker build -t <name> .
```

## Usage

```
user@machine$: docker run -v <local_output_dir>:<docker_output_dir> -v <local_import_dir>:<docker_import_dir> --rm <image:latest> <themes> --dir-output <docker_output_dir> <docker_import_dir/file>
```


## Usage

```
Convert your favorite images and wallpapers with your favorite color palettes/themes

Usage: dipc [OPTIONS] <PALETTE> [FILE]...

Arguments:
  <PALETTE>
          The color palette to use:
              - name of a builtin theme
              - path to a theme in JSON
              - a JSON string with the theme (starting with `JSON: {}`)
          Run with --help instead of -h for a list of all builtin themes

          Builtin themes:
              - catppuccin
              - dracula
              - edge
              - everforest
              - gruvbox
              - gruvbox-material
              - nord
              - onedark
              - rose-pine
              - solarized
              - tokyo-night

  [FILE]...
          The image(s) to process

Options:
  -s, --styles <VARIATIONS>
          The color palette variation(s) to use
          Run with --help instead of -h for a list of all possible values

          Possible values:
              - `all` to generate an image for each of the variations
              - `none` if you are using a flat theme without variations
              - or a comma-delimited list of the names of variations it should use

          [default: all]

  -o, --output <PATH>
          Output image(s) name/path as a comma-delimited list

  -d, --dir-output <PATH>
          Output directory name/path

  -m, --method <METHOD>
          CIELAB DeltaE method to use

          [default: de2000]

          Possible values:
          - de2000:  The default DeltaE method
          - de1994g: CIE94 DeltaE implementation, weighted with a tolerance for graphics
          - de1994t: CIE94 DeltaE implementation, weighted with a tolerance for textiles
          - de1976:  The original DeltaE implementation, a basic euclidian distance formula

  -v, --verbose...
          Verbose mode (-v, -vv, -vvv)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Convert all images in directory

```sh
# Save to current directory
dipc <PALETTE> <INPUT_DIR>/*

# Save to output directory
dipc --dir-output <PATH> <PALETTE> <INPUT_DIR>/*
```

### Convert multiple images

```sh
dipc <PALETTE> img0.png img1.png

# Rename files
dipc --output new-img0.png,new-img1.png <PALETTE> img0.png img1.png
```

### Color palette variation(s)/style(s)

```sh
dipc --styles Style0 <PALETTE> img.png
dipc --styles Style0,Style1 <PALETTE> img.png
```

### CIELAB DeltaE method

```sh
dipc --method <METHOD> <PALETTE> img.png
```

## License

`dipc` is dual-licensed under the terms of both the MIT License and the Apache License 2.0

SPDX-License-Identifier: MIT OR Apache-2.0
