# river-luatile

This is a little program that lets you write your own
[river](https://github.com/riverwm/river) layout generator in lua.

See the example (and the default) layout
[here](https://github.com/MaxVerevkin/river-luatile/blob/master/layout.lua). The
example should be self-explanatory™.

The layout must be located at `$XDG_CONFIG_HOME/river-luatile/layout.lua` or
`~/.config/river-luatile/layout.lua`.

The layout namespace (for now) is always `luatile`.

Your layout must at least implement the function `handle_layout()` and
optionally a function `handle_metadata()`, which will be used to query
metadata about the layout. Currently, the layout name is the only metadata
supported, see the example.

## Installation

[![Packaging status](https://repology.org/badge/vertical-allrepos/river-luatile.svg)](https://repology.org/project/river-luatile/versions)

### From source

```sh
cargo install river-luatile --locked
```
