# Fuck-Jit - An LLVM Brainfuck JIT using inkwell
[![Rust](https://github.com/4gboframram/Fuck-JIT/actions/workflows/rust.yml/badge.svg)](https://github.com/4gboframram/Fuck-JIT/actions/workflows/rust.yml)
This was a little LLVM brainfuck JIT that I made to start learning LLVM and to get used to Rust

`fuck-jit` can do Just-in-time or static compilation of brainfuck code. It can also generate a readable assembly file.

## Building
### Unix
#### Before Building

Make sure you have the following:
- `nix` package manager


#### Building
First, set up the nix environment by running
- `nix-shell`

Then, in that new environment run cargo to build the executable. It may take a while to compile all of the dependecies, so you should probably go get yourself some popcorn.

- `cargo build --release`


If the build fails, the version of the rust toolchain on your system's nixpkgs may be outdated, 
and the dependency `clap` requires cargo version `1.54.0` or higher. 

I provided an extra nix environment that could be used in this case that does not rely on overlays, but it should not be used unless you have to because it actually does install rust globally while in the shell. You can repeat the earlier steps, but running `nix-shell outdated_rust.nix` instead of just `nix-shell`

Please go easy on me, I'm still learning how to use Nix.


### Windows (Does not exist yet)
- I'm still trying to figure out how to get llvm to work as a dependency on Windows

## Usage:
```
USAGE:
    brainfuck-jit [OPTIONS] --infile <INFILE>

OPTIONS:
    -a, --asm                    Compile Brainfuck to an assembly file
    -h, --help                   Print help information
    -i, --infile <INFILE>        Input file for the compiler.
        --ir                     Print (unoptimized) LLVM IR generated from the input to stderr.
    -o, --outfile <OUTFILE>      Compile Brainfuck to an object or assembly file
    -t, --tape-len <TAPE_LEN>    Tape length for the compiler [default: 30000]
    -V, --version                Print version information
```

## TODO
- Provide Windows binaries
- Provide a build tutorial for Windows
- Optimize common Brainfuck idioms
  

If you are reading this, I just want to say that LLVM is a pain in the ass to use as a dependency and that you should not use it unless you absolutely need it. 

Also don't judge me for using replit. It is my only option for working on projects while at school
