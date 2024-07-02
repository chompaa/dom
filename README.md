# Dom

> A scripting language written in Rust

## Playground

![playground](https://github.com/chompaa/dom/assets/26204416/da683a18-ae99-45cd-8023-d3350f127543)

You can try Dom for yourself using the playground [here](https://chompaa.github.io/dom/).

## Features 

- [x] Comments
- [ ] Types
    - [x] Booleans
    - [x] Integers
    - [ ] Floats
    - [x] Strings
    - [x] Lists
- [ ] Variables
    - [x] Mutable
    - [ ] Constant
- [x] Comparisons
- [x] Unary expressions
- [x] Binary expressions
- [x] Scope
- [x] Functions
    - [x] Defined
    - [x] Built-in
- [x] Loops
- [ ] Control flow
    - [x] Conditional statements
        - [x] Single conditions
        - [x] Multiple conditions
    - [x] Return
    - [x] Continue
    - [x] Break

## Syntax

### Comments

Commenting is done with the `//` characters:

```rs
// Hello, world!
```

They are not parsed in any manner.

### Comparison

Numerical comparison uses the usual operators:

```js
let foo = 1
let bar = 2

print(foo <= bar) // true
print(foo >= bar) // false
print(foo != bar) // true
print(foo == bar) // false
```

Likewise, for binary comparisons:

```js
let foo = true
let bar = false

print(foo && bar) // false
print(foo || bar) // true
```

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

### Lists

Lists can be created using brackets `[..]`:

```rust
let list = [0, "foo", 1, "bar"]
print(get(list, 1)) // "foo"
```

There are built-in functions for working with lists: `get`, `set`, `push`, `pop`, and `len`.

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

Dom has support for pipes, which let you pass the result of one function onto the next. For example:

```elixir
["foo"]
|> push("bar")
|> print() // ["foo", "bar"]
```

Dom also contains some built-in functions, which can be seen below:

> [!NOTE]
> These functions don't produce errors right now, i.e. for incorrect arguments or runtime errors.

| Function | Arguments | Description |
| --- | --- | --- |
| `print` | `Any` | Outputs a literal to the console | |
| `input` | `None` | Requests and returns input from the console. Not supported in `dom_wasm` |
| `get` | `List, Int` | Gets an item at a specified index from a `List` 
| `set` | `List, Int, Any` | Sets an item at a specified index in a `List` 
| `push` | `List, Any` | Pushes an item to the end of a `List` 
| `pop` | `List, Int` | Pops an item at a specified index in a `List` 
| `len` | `List, Int` | Returns the length of a `List` 


### Loops

Loops are defined using the `loop` keyword, and use `break` and `continue` for control flow:

```rs
let foo = 0
let bar = 2

loop {
    foo = foo + 1
    if foo == bar {
        // Exit this iteration 
        continue
    }
    print(foo)
    if foo > 10 {
        // Exit the loop
        break
    }
}
```

</details>

## Running locally 

Make sure you have the Rust toolchain installed.

- Clone this repository and navigate to it:

```sh
git clone https://github.com/chompaa/dom && cd dom
```

- To start the interactive shell:

```sh
cargo run -p dom_cli
```

- To interpret a file:

```sh
cargo run -p dom_cli <file>
```

