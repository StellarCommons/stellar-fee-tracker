# stellar-devkit

Developer toolkit for testing and simulating the Stellar fee tracker.

This crate was reset and is being rebuilt from scratch. Work is tracked as a sequence of
issues labeled `devkit`, titled "Devkit #1" through "Devkit #125", each building on the one
before it, starting from this foundation.

## Command-line interface

```bash
cargo run -p stellar-devkit -- --help
cargo run -p stellar-devkit -- version
```

`version` prints the crate version and the build metadata of the binary. The other
subcommands — `replay`, `convert`, `export`, `benchmark`, and `config` — are wired up but
still placeholders until their issues land.

## Development

```bash
cargo build -p stellar-devkit
cargo test -p stellar-devkit
cargo clippy -p stellar-devkit --all-targets --all-features -- -D warnings
cargo fmt --check -p stellar-devkit
```
