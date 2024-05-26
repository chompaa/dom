# Dom

> An experimental scripting language with minimalism in mind

## Features 

- [ ] Types
    - [x] Booleans
    - [x] Integers
    - [ ] Floats
    - [x] Strings
- [x] Mutable variables
- [x] Binary operations/expressions
- [ ] Conditional statements
- [x] Functions
- [ ] Loops

## Syntax

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
- Less than (or eq) `<(=)`
- Greater than (or eq)`>(=)`

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

### Functions

Functions are defined using the `fn` keyword as follows:

```rs
fn sum(a, b) {
    a + b
}
```

They always return the last evaluated expression (there is no way to return early, yet). They are called as you may expect:

```rs
sum(1, 1)
```

Arguments are always passed by value, for now.

Dom also contains some built-in functions, which can be seen below:

| Function | Arguments | Example | Description |
| --- | --- | --- | --- |
| `print` | `Int \| Str` | `print("Hello, world")` | Outputs a literal to the console

</details>

## Running 

Make sure you have the Rust toolchain installed.

- Clone this repository and navigate to it:

```sh
git clone https://github.com/chompaa/dom && cd dom
```

- To start the interactive shell:

```sh
cargo run
```

- To interpret a file:

```sh
cargo run <file>
```

