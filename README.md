# rust-life
A rather bad implementation of Conway's game of life in rust.

### usage:
To play the game you first need to build it,<br>
then just go to `target/release` and run `rust-life`.

(or do this: `$ ./target/release/rust-life`)

#### controlls:
Space to start/pause.<br>
Q to quit.<br>
The mouse to place cells.<br>

#### misc:
You can load life 1.06 compatable files by using the `-l` flag.<br>
ex.:<br>
`$ rust-life -l glider.lif`

To change time it takes for one tick to pass use the `-t` flag.<br>
ex.:<br>
`$ rust-life -t 0.3`

To change the rules of the game use either the `-s` flag to change how many neighbors a cell needs 
to survive or the `-n` flag to change how many neighbors a new cell needs to have to spawn.<br>
ex.:<br>
`$ rust-life -s 2, 3 -n 3`

If things are still unclear, refer to the help menu.<br>
ex.:<br>
`$ rust-life -h`

### building instructions:
Pull the repository and do:<br>
`$ cargo build --release`<br>

That's it.

### libs used:
* [termion](https://github.com/ticki/termion)
* [getopts](https://doc.rust-lang.org/getopts/getopts/)
