# Loisp

Low Level Lisp (Loisp) is an S-expression based programming language implemented in [Rust](https://rust-lang.org).

## Quick Start

To compile the generated assembly, the compiler uses the [flat assembler](https://flatassembler.net/), so you will need to have it installed:

```console
$ echo "(print (+ 34 35))" > test.loisp
$ cargo run -- run test.loisp
```

## This language is planned to be

- [x] Compiled
- [x] Native
- [ ] Turing Complete (yes, the language is in such a early state, that it isn't turing complete yet)
- [ ] Interpreted
- [ ] Interactive (by implementing a [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop))
- [/] Type Checked
- ___ Cross-Platform (maybe in the future when i have a better computer)

## Documentation

To read the documentation, see [docs.md](./docs/docs.md).
