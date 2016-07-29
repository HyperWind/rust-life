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

To change the time it takes for one tick to pass use the `-t` flag.<br>
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
 
### license:

Copyright (c) 2016 Karolis Lasys \<<karolioofficial@outlook.com>\>
<br><br>
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
<br><br>
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
<br><br>
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


