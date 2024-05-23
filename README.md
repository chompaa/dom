# Dom

> An experimental scripting language with minimalism in mind

## Features 

- [ ] Types
    - [x] Integers
    - [ ] Floats
    - [ ] Strings
- [x] Mutable variables
- [x] Binary operations/expressions
- [ ] Conditional statements/closures
- [x] Functions
- [ ] Loops

## Syntax

### Arithmetic

Arithmetic can be performed as you would expect. For example:

```rs
(2 + 2) * (2 / 2) - 2
```

Outputs `2`. Operations follow the usual order of operations.

### Variables

Variables can be set using the `let` keyword as follows:

```rs
let foo = 1
```

They are always mutable.

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
cargo run -- <file>
```

