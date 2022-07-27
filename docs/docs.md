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

- Polish notation: https://en.wikipedia.org/wiki/Polish_notation
- S-expressions: https://en.wikipedia.org/wiki/S-expression

## Instructions

The available instructios are the following:

| Instruction         | Purpose                                                                                                                                 |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| `print`             | Prints the value given as parameter                                                                                                     |
| `+`                 | Return the sum of two values given as parameters                                                                                        |
| `-`                 | Return the difference between two values given as parameter                                                                             |
| `*`                 | Return the result of the multiplication between the parameters                                                                          |
| `/`                 | Return the result of the division between the parameters                                                                                |
| `%`                 | Return the rest of the division between the parameters                                                                                  |
| `syscall`           | Perform a syscall with the first parameter being the number of the syscall and the rest of the parameters being passed to the syscall   |
| `setvar`            | Create a variable with the name given as first parameter and set the value of that variable to the value of the second parameter        |
| `getvar`            | Return the value of the variable with name given as first parameter                                                                     |
| `chvar`             | Change the value of an variable to the specified one                                                                                    |
| `while`             | Keeps executing the instructions given as parameters until the first parameter (condition) returns 0                                    |
| `if`                | Use the first parameter as a condition, if the condition returns 1, it executes the block given as 1st parameter, else the 2nd one      |
| `block`             | Just executes all the instructions given as parameters                                                                                  |
| `=`                 | Compare 2 parameters and return 1 if they are equal                                                                                     |
| `!=`                | Compare 2 parameters and return 1 if they are not equal                                                                                 |
| `<`                 | Compare 2 parameters and return 1 if the first is less than the second                                                                  |
| `>`                 | Compare 2 parameters and return 1 if the first is greater than the second                                                               |
| `<=`                | Compare 2 parameters and return 1 if the first is less than the second or if they are equal                                             |
| `>=`                | Compare 2 parameters and return 1 if the first is greater than the second or if they are equal                                          |
| `ptrto`             | Return a pointer to the variable that has the name given as first parameter                                                             |
| `load64`            | Return a value by dereferecing the pointer given as first parameter                                                                     |
| `store64`           | Store the value given as second parameter into the pointer that was given as first parameter                                            |
| `load32`            | Return a 32 bit value by dereferecing the pointer given as first parameter                                                              |
| `store32`           | Store a 32 bit value given as second parameter into the pointer that was given as first parameter                                       |
| `load16`            | Return a 16 bit value by dereferecing the pointer given as first parameter                                                              |
| `store16`           | Store a 16 bit value given as second parameter into the pointer that was given as first parameter                                       |
| `load8`             | Return a 8 bit value by dereferecing the pointer given as first parameter                                                               |
| `store8`            | Store a 8 bit value given as second parameter into the pointer that was given as first parameter                                        |
| `alloc`             | Allocate a memory buffer with the capacity given as second parameter and give it a name (first parameter)                               |
| `getmem`            | Return a pointer to a memory buffer that has the name given as first parameter                                                          |
| `<<`                | Perform the 'shift left' operation in the parameters and return the result                                                              |
| `>>`                | Perform the 'shift right' operation in the parameters and return the result                                                             |
| `&`                 | Perform the 'and' operation in the parameters and return the result                                                                     |
| <code>&#124;</code> | Perform the 'or' operation in the parameters and return the result                                                                      |
| `!`                 | Perform the 'not' operation in the parameters and return the result                                                                     |
| `macro`             | Create a macro with a body that contains the instructions given as parameters                                                           |
| `expand`            | Expand the macro that has the name given as parameter                                                                                   |
| `pop`               | Pop an element from the runtime stack and store it in the variable that has the name given as first parameter                           |
| `castint`           | Return the value given as parameter with type `Integer`                                                                                 |
| `castptr`           | Return the value given as parameter with type `Pointer`                                                                                 |
| `include`           | Compile a file given as parameter and bring everything from that file into the current scope                                            |
| `defun`             | Create a function with the name given as first parameter that executes the instructions given as rest of the parameters                 |
| `call`              | Call a function with the name given as first parameter. If the user provide more parameters, they will be pushed into the runtime stack |
| `increment`         | See [Enumerations](#Enumerations)                                                                                                       |
| `reset`             | See [Enumerations](#Enumerations)                                                                                                       |

## Enumerations

There are two instructions available for enumerations, `increment` and `reset`. They work the following way, at compile time, the compiler maintain a counter called `iota` (just like Go's iota), `increment` will return the internal `iota` and then increment it by the value given as parameter, and `reset` will return the internal `iota`, and reset its value. Combining `increment`, `reset` and macros, you can create enumerations.

### Example
```lisp
(macro MONDAY    (increment 1))
(macro TUESDAY   (increment 1))
(macro WEDNESDAY (increment 1))
(macro THURSDAY  (increment 1))
(macro FRIDAY    (increment 1))
(macro SATURDAY  (increment 1))
(macro SUNDAY    (reset))
```

## Control Flow

### Conditions:

```lisp
(if <condition>
  <then>
  <else>
)
```

#### Example

```lisp
(if 1
  (print 10)
  (block)
)
```
Output:

```console
10
```

### Loops:

```lisp
(while <condition>
  <body>
)
```
#### Example

```lisp
(setvar x 0)

(while (!= (getvar x) 10)
  (print (getvar x))
  (chvar x (+(getvar x)1))
)
```
Output:
```console
0
1
2
3
4
5
6
7
8
9
```
