# Contributing to skill-issue

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

1. Install [Rust](https://rustup.rs/) (stable toolchain)
2. Clone the repo and build:

```bash
git clone https://github.com/daviddrummond95/skill-issue-cli.git
cd skill-issue
cargo build
```

3. Run tests:

```bash
cargo test
```

## Making Changes

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass
4. Submit a pull request

## Adding Rules

Rules are defined as regex patterns in `patterns/*.toml`. Each rule needs:

- A unique ID (e.g., `SL-NET-005`)
- A severity level (`info`, `warning`, or `error`)
- A regex pattern (note: Rust's `regex` crate does **not** support lookahead/lookbehind)
- A human-readable description and recommendation

See existing pattern files for examples.

## Code Style

- Run `cargo fmt` before committing
- No clippy warnings (`cargo clippy -- -D warnings`)
- Keep changes focused — one concern per PR

## Issues

Found a bug or have a feature request? [Open an issue](https://github.com/daviddrummond95/skill-issue-cli/issues). For security issues, see [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
