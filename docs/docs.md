# Documentation

## Syntax

Loisp (like every lisp language) is S-expression based and uses polish notation, details [here](https://en.wikipedia.org/wiki/Polish_notation). So the syntax will look like this:

*sum two numbers and print them*:
```lisp
(print (+ 34 35))
```

The fact that this language is S-expression based is given by the fact that, the instruction name and parameters are passed through elements in a "list". And the Polish notation is given by the fact that the style of operations are made as the following code:

```lisp
(+ 34 35)
```

### References

- Reverse polish notation: https://en.wikipedia.org/wiki/Polish_notation
- S-expressions: https://en.wikipedia.org/wiki/S-expression

## Instructions

The available instructios are the following:

| Instruction | Purpose                                                     |
|-------------|-------------------------------------------------------------|
| `print`     | Prints the value given as parameter                         |
| `+`         | Return the sum of two values given as parameters            |
| `-`         | Return the difference betwenn two values given as parameter |
