# river-luatile

This is a little program that lets you write your own [river](https://github.com/riverwm/river) layout generator in lua.

See the example (and the default) layout [here](https://github.com/MaxVerevkin/river-luatile/blob/master/layout.lua). The example should be self-explanatory™.

The layout must be located at `$XDG_CONFIG_HOME/river-luatile/layout.lua` or `~/.config/river-luatile/layout.lua`.

The layout namespace (for now) is always `luatile`.

## Installation

### Arch/AUR

<https://aur.archlinux.org/packages/river-luatile-git>

### NixOS

Use the provided flake package or overlay.

### Manually from source

```sh
git clone https://github.com/MaxVerevkin/river-luatile
cd river-luatile
cargo install --path . --locked
```
