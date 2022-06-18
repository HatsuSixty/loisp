# Loisp

Low Level Lisp (Loisp) is an S-expression based programming language implemented in [Rust](https://rust-lang.org).

## Quick Start

For now, programs written in this language are read from a file in the same working directory of the compiler called `test.loisp`. To compile the generated assembly, the compiler uses the [yasm](https://yasm.tortall.net/) assembler, so you will need to have it installed:

```console
$ echo "(print (+ 34 35))" > test.loisp
$ cargo run
$ ./output
```

## Documentation

To read the documentation, see [docs.md](./docs/docs.md).
