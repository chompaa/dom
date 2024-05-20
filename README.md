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
- [ ] Functions
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

## Running 

> [!NOTE]
> This language is barebones at the moment. As such, there is a usable shell to test syntax, but no support for file reading.

Make sure you have the Rust toolchain installed.

1. Clone this repository:

```sh
git clone https://github.com/chompaa/dom
```

2. Run:

```sh
cargo run
```

