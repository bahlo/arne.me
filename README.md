# arne.me [![nix build](https://github.com/bahlo/arne.me/actions/workflows/nix_build.yml/badge.svg)](https://github.com/bahlo/arne.me/actions/workflows/nix_build.yml)

My personal website.

## Development

### With Nix (recommended)

Make sure you have [Nix](https://nixos.org) set up.
Run `nix develop` (or `nix-shell`, if you don't have Flakes enabled) to get all
dependencies with the same versions that are used on CI.

### Without Nix

If you don't use Nix, head over to the [Installation docs of Zola](https://www.getzola.org/documentation/getting-started/installation/),
the static site generator this project uses, and install it on your system.
For the scripts you'll need `python3` with the packages `colormath` and `toml` installed.  

### Running

Run `zola serve` to start a development server with hot reloading enabled.