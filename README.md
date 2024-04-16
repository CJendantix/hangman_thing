# A Hangman Game by CJendantix
I made this because I was bored I guess<br>
I got hyperfocused<br>
It kinda sucks<br>
But also doesn't<br>
...<br>
anyways<br>

## Usage
Always best to run new programs with `--help` first but I'll document it here anyways
This program is special because:
1. You can make your own word lists
2. You can put limiters for the length of the words you want in that word list
3. You can choose how many chances you get
anyways
```
Options:
  -f, --filename <FILENAME>                    [default: ./words.txt]
  -g, --wrong_guesses <WRONG_GUESSES_ALLOWED>  [default: 8]
  -l, --word_length <WORD_LENGTH_RANGE>        [default: 3:8]
```
pretty self-explanatory
#### To the toki ponists
The build directory contains a toki pona word list at `word_lists/tp.txt`, so run the program with `-f ./word_lists/tp.txt` to play in toki pona mode

## Build instructions
Install [rustup](https://rustup.rs/)
```bash
rustup default stable
git clone https://github.com/CJendantix/rust_hangman.git
cd rust_hangman
cargo build --release
```
The binary is now located at `./target/release/rust_hangman`
