# naaw

[bspwm](https://github.com/baskerville/bspwm) node tagger, [maaw](https://gitlab.com/samaingw/maaw)'s little brother.

## Usage

`naaw server <x>`: start the naaw server, tagged nodes will get a border width of <x> pixels

`naaw tag <n>`: toggle the tagged/untagged status of the node with id <n> (e.g. `naaw tag $(bspc query -N -n)` to toggle the current node)

`naaw show`: show/hide the currently tagged nodes
