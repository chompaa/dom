# Dom

> A scripting language written in Rust

## Playground

![playground](https://github.com/chompaa/dom/assets/26204416/3cef6cb1-3ff4-4fc9-999d-7c828b28197d)

You can try Dom for yourself using the playground [here](https://chompaa.github.io/dom/).

## Features 

- [x] Comments
- [ ] Types
    - [x] Booleans
    - [x] Integers
    - [ ] Floats
    - [x] Strings
- [x] Mutable variables
- [x] Binary operations/expressions
- [ ] Conditional statements
    - [x] Single conditions
    - [ ] Multiple conditions
- [x] Functions
    - [x] Returns
- [x] Loops

## Syntax

### Comments

Commenting is done with the `//` characters:

```rs
// Hello, world!
```

They are not parsed in any manner.

### Comparison

Comparison uses the usual operators:

```js
let foo = 1
let bar = 2

print(foo <= bar) // true
print(foo >= bar) // false
print(foo != bar) // true
print(foo == bar) // false
```

Currently supported operations are:
- Equal `==`
- Not equal `!=`
- Less than `<`
- Less than or equal `<=`
- Greater than `>`
- Greater than or equal `>=`

### Arithmetic

Arithmetic can be performed as you would expect. For example:

```rs
(2 + 2) * (2 / 2) - 2
```

Outputs `2`. Operations follow the usual order of operations.

Currently supported operations are:
- Addition `+`
- Subtraction `-`
- Multiplication `*`
- Division `/`

### Variables

Variables can be set using the `let` keyword as follows:

```rs
let foo = 1
```

They are always mutable.

</details>

### Conditionals

Conditional statements can be formed using the `if` keyword:

```rs
let foo = 1
let bar = 2

if foo < bar {
    print("`foo` is less than `bar`")
}
```

### Functions

Functions are defined using the `fn` keyword as follows:

```rs
fn sum(a, b) {
    a + b
}
```

Unless the `return` keyword is used, they return the last evaluated expression. They are called as you may expect:

```rs
sum(1, 1)
```

Arguments are always passed by value, for now.

Dom also contains some built-in functions, which can be seen below:

| Function | Arguments | Example | Description |
| --- | --- | --- | --- |
| `print` | `Int \| Str` | `print("Hello, world")` | Outputs a literal to the console

### Loops

Loops are defined using the `loop` keyword, and use `break` and `continue` for control flow:

```
rs
let foo = 0
let bar = 2

loop {
    if foo == bar {
        // Exit this iteration 
        continue
    }

    print(foo)
    foo = foo + 1

    if foo > 10 {
        // Exit the loop
        break
    }
}
```

</details>

## Interpreting 

Make sure you have the Rust toolchain installed.

- Clone this repository and navigate to it:

```sh
git clone https://github.com/chompaa/dom && cd dom/dom
```

- To start the interactive shell:

```sh
cargo run
```

- To interpret a file:

```sh
cargo run <file>
```

