# Loisp

Low Level Lisp (Loisp) is an S-expression based programming language implemented in [Rust](https://rust-lang.org).

## Quick Start

To compile the generated assembly, the compiler uses the [flat assembler](https://flatassembler.net/), so you will need to have it installed:

```console
$ echo "(print (+ 34 35))" > test.loisp
$ cargo run -- run test.loisp
```

## Testing

The compiler has a tester built-in to it, so you can use that to test if all the features are working properly. To use the built-in tester, there is a subcommand to use it:

```console
$ cargo run -- run-test tests # for more details, see `cargo run -- help`
```

## This language is planned to be

- [x] Compiled
- [x] Native
- [x] Useful (that basically means that the language has enough features to create useful applications)
- [x] Turing Complete (see [./examples/rule110.loisp](./examples/rule110.loisp))
- [x] Interpreted
- [x] Interactive (by implementing a [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop))
- [x] Type Checked
- Cross-Platform (maybe in the future when i get a better computer)

## Documentation

To read the documentation, see [docs.md](./docs/docs.md).
