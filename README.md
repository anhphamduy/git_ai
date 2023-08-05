# Intelligent Git Generator

The Intelligent Git Generator (also known as `git_ai`) is an intuitive AI tool designed to streamline your git version control system. It not only provides intelligent suggestions for commit messages but also assists in creating comprehensive PR templates.

## Installation

The Intelligent Git Generator is written in Rust. You can either build from source or download the binary.

### Downloading the Binary

You can also download the precompiled binary from this [link](https://example.com/download).

### Building from Source

Ensure that you have Rust installed on your machine. You can check this by running:

```bash
rustc --version
```

If Rust is not installed, follow [the instructions](https://www.rust-lang.org/tools/install) to install it.

Once Rust is installed, clone this repository, navigate to the directory, and build the project:

```bash
git clone https://github.com/anhphamduy/git_ai
cd git_ai
cargo build --release
```

The built binary will be located in the `target/release/` directory.

## Usage

Use the `git_ai` command followed by a subcommand. For example:

```bash
git_ai commit
```

This will suggest a commit message based on your changes.

### Commands

The Intelligent Git Generator supports the following commands:

1. `commit` - Suggests a commit message based on your changes.
2. `pr` - Generates a PR template for your current branch.
3. `init` - Sets up the tool in your local environment.

### Subcommand Usage

#### commit

Use the `commit` subcommand to get suggestions for commit messages. 

Usage:

```bash
git_ai commit [OPTIONS]
```

Options:

- `-m, --message <MESSAGE>` - Provides the context for the commit.
- `-n, --name-only` - Suggests a commit message based only on the names of the changed files.

#### pr

Use the `pr` subcommand to get a suggested PR template.

Usage:

```bash
git_ai pr [OPTIONS] [BRANCH]
```

Arguments:

- `[BRANCH]` - The branch to be PR'ed in. The default is `main`.

Options:

- `-m, --message <MESSAGE>` - Provides the context for the PR.

## License

This project is licensed under [MIT License](./LICENSE).