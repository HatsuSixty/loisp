# Loisp

Low Level Lisp (Loisp) is an S-expression based programming language implemented in [Rust](https://rust-lang.org).

## Quick Start

For now, programs written in this language are read from a file in the same working directory of the compiler called `test.loisp`:

```console
$ echo "(print (+ 34 35))" > test.loisp
$ cargo run
$ nasm -felf64 output.asm
$ ld -o output output.o
$ ./output
```

## Documentation

To read the documentation, see [docs.md](./docs/docs.md).
