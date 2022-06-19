# Loisp

Low Level Lisp (Loisp) is an S-expression based programming language implemented in [Rust](https://rust-lang.org).

## Quick Start

To compile the generated assembly, the compiler uses the [yasm](https://yasm.tortall.net/) assembler, so you will need to have it installed:

```console
$ echo "(print (+ 34 35))" > test.loisp
$ cargo run -- run output
```

## Documentation

To read the documentation, see [docs.md](./docs/docs.md).
