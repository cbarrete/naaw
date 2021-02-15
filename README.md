# naaw

[bspwm](https://github.com/baskerville/bspwm) node tagger, [maaw](https://gitlab.com/samaingw/maaw)'s little brother.

## Usage

`naaw server <x>`: start the naaw server, tagged nodes will get a border width of <x> pixels

`naaw tag <n>`: toggle the tagged/untagged status of the node with id <n> (e.g. `naaw tag $(bspc query -N -n)` to toggle the current node)

`naaw show`: show/hide the currently tagged nodes

## Installation & example config

`cargo install --path .`

```
# in ~/.config/sxhkd/sxhkdrc

# toggle tag on current node
super + i
    naaw tag $(bspc query -N -n)

# toggle tag visibility
super + u
    naaw show


# in ~/.config/bspwm/bspwmrc

naaw server 5 2>/dev/null &
```
