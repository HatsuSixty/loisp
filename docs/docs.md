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

| Instruction | Purpose                                                                                                                               |
|-------------|---------------------------------------------------------------------------------------------------------------------------------------|
| `print`     | Prints the value given as parameter                                                                                                   |
| `+`         | Return the sum of two values given as parameters                                                                                      |
| `-`         | Return the difference between two values given as parameter                                                                           |
| `*`         | Return the result of the multiplication between the parameters                                                                        |
| `/`         | Return the result of the division between the parameters                                                                              |
| `%`         | Return the rest of the division between the parameters                                                                                |
| `syscall`   | Perform a syscall with the first parameter being the number of the syscall and the rest of the parameters being passed to the syscall |
| `setvar`    | Create a variable with the name given as first parameter and set the value of that variable to the value of the second parameter      |
| `getvar`    | Return the value of the variable name given as first parameter                                                                        |
| `chvar`     | Change the value of an variable to the specified one                                                                                  |
| `while`     | Keeps executing the instructions given as parameters until the first parameter (condition) returns false                              |
