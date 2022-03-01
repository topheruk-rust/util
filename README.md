# rust

Rust Playground

## workspaces

- Create directory

```
$ mkdir add cd <PACKAGE_NAME>
$ cd <PACKAGE_NAME>
```

### Creating a binary package

- Edit `Cargo.toml`

```
[workspace]

members = [
  "PACKAGE_NAME"
]
```

- Create binary for package:

```
$ cargo new <PACKAGE_NAME>
```

- Build workspace using `cargo build`. Files should look like this:

```
Cargo.lock
Cargo.toml
<PACKAGE_NAME>/
  Cargo.toml
  src/
    main.rs
target/
```

### Creating a libary

- amend `[workspace]:`

```
members = [
  "PACKAGE_NAME",
  "LIBRARY_NAME",
]
```

- Generate a new library

```
$ cargo new <LIBRARY_NAME> --lib
```

- Project structure should look like this now:

```
Cargo.lock
Cargo.toml
<PACKAGE_NAME>/
  Cargo.toml
  src/
    main.rs
<LIBRARY_NAME>/
  Cargo.toml
  src/
    lib.rs
target/
```

## Using `lib` in `bin` project

- to link library, add to `[dependencies]` of binary project `Cargo.toml:`

```
[dependencies]
<LIBRARY_NAME> = { path = RELATIVE_PATH }
```

- To execute binary project: `$ cargo build` & `$ cargo run -p <BINARY_NAME>`

- `cargo run --bin [binary] [args]`
