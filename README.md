# Development container isolation

Opinionated setup helper for [**containers.dev**](https://containers.dev/) to mount persistent folders and predefined settings.

Generate dev container environment with an interactive prompt and mounted shared directories to have persistent vscode extensions.
Try run can be used to preview everything without creating any persistent data.

Features:

- Interactive prompt
- Supported Languages
  - Rust
  - .NET
  - Node.js
- Generate Files
  - Containerfile
  - devcontainer.json
- Create and Mount persistent directories for vscode extensions

## Build

Install [Rust](https://rust-lang.org/) and Cargo with support for the Rust 2024 edition (Rust 1.85 or newer).

From the project root, build the release binary:

```sh
cargo build --release
```

The executable is written to `target/release/dev-container-isolation` (`dev-container-isolation.exe` on Windows).

## Run

Run the helper against an existing repository:

```sh
cargo run -- --target-path /path/to/repository
```

If `--target-path` is omitted, the helper uses the current directory.

Preview the setup without creating files or directories:

```sh
cargo run -- --target-path /path/to/repository --try-run true
```

Use `--file-override true` to replace existing configuration files. Boolean options take an explicit `true` or `false` value. View all options with:

```sh
cargo run -- --help
```

## License

Licensed under the [MIT License](LICENSE).
