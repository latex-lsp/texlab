[![CI](https://github.com/latex-lsp/texlab/workflows/CI/badge.svg)](https://github.com/latex-lsp/texlab/actions)
[![Coverage](https://codecov.io/gh/latex-lsp/texlab/branch/master/graph/badge.svg)](https://codecov.io/gh/latex-lsp/texlab)
[![Rust](https://img.shields.io/badge/rustc-1.39%2B-blue)](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html)
[![Dependabot](https://api.dependabot.com/badges/status?host=github&repo=latex-lsp/texlab)](https://dependabot.com)

[![GitHub release](https://img.shields.io/github/release/latex-lsp/texlab?label=github)](https://github.com/latex-lsp/texlab/releases)
[![CTAN](https://img.shields.io/ctan/v/texlab)](https://ctan.org/pkg/texlab)
[![Arch Linux](https://repology.org/badge/version-for-repo/arch/texlab.svg?header=arch%20linux)](https://www.archlinux.org/packages/community/x86_64/texlab/)
[![NixOS Stable](https://repology.org/badge/version-for-repo/nix_stable/texlab.svg?header=nixos%20stable)](https://nixos.org/nixos/packages.html?channel=nixos-20.03&query=texlab)
[![NixOS Unstable](https://repology.org/badge/version-for-repo/nix_unstable/texlab.svg?header=nixos%20unstable)](https://nixos.org/nixos/packages.html?channel=nixpkgs-unstable&query=texlab)
[![Homebrew](https://repology.org/badge/version-for-repo/homebrew/texlab.svg?header=homebrew)](https://formulae.brew.sh/formula/texlab)
[![Scoop](https://repology.org/badge/version-for-repo/scoop/texlab.svg?header=scoop)](https://scoop.sh/)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
We provide an [extension](https://github.com/latex-lsp/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

Learn more about the project on our [website](https://texlab.netlify.app).

## Getting Started

See the [installation chapter](https://texlab.netlify.app/docs) from our docs.

## Building from Source

You will need to install the following dependencies to compile the server:

- [Rust (>= 1.39)](https://rustup.rs/)

Then run the following command in the project folder:

```shell
cargo build --release
```

Alternatively, Rust users can run the following command
without having to clone this repository:

```shell
cargo install --git --locked https://github.com/latex-lsp/texlab.git
```

## Development

You can create a debug build by building the server without the `--release` flag.
The resulting build can be used with the [Visual Studio Code extension](https://github.com/latex-lsp/texlab-vscode)
by adding the absolute path of the `target/debug` folder to your `PATH` environment variable.

TexLab has an extensive test suite of unit and integration tests. You can run them by executing

```shell
cargo test
```

in the project folder.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
