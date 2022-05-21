# ClipCC VM in Rust

A expermental vitrual machine for ClipCC. Written in Rust.

## Build / Run mininal example

```bash
cargo build --release --example mininal -- <SB3_PATH>
cargo run --release --example mininal -- <SB3_PATH>
```

## Features

- Load sb3 projects
- Fast interpreter (no JIT but still faster than the original Scratch VM)

## TODO

- Add WASM support
- Integrate with ClipCC
