[![CI](https://github.com/latex-lsp/texlab/workflows/CI/badge.svg)](https://github.com/latex-lsp/texlab/actions)
[![Coverage](https://codecov.io/gh/latex-lsp/texlab/branch/master/graph/badge.svg)](https://codecov.io/gh/latex-lsp/texlab)
[![Rust](https://img.shields.io/badge/rustc-1.58.1%2B-blue)](https://blog.rust-lang.org/2022/01/20/Rust-1.58.1.html)

[![GitHub release](https://img.shields.io/github/release/latex-lsp/texlab?label=github)](https://github.com/latex-lsp/texlab/releases)
[![crates.io](https://img.shields.io/crates/v/texlab)](https://crates.io/crates/texlab)
[![CTAN](https://img.shields.io/ctan/v/texlab)](https://ctan.org/pkg/texlab)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
The server may be used with [any editor that implements the Language Server Protocol](https://microsoft.github.io/language-server-protocol/implementors/tools/).

![Demo](docs/demo.gif)

## Getting Started

If your editor extension like does not install the TexLab server automatically,
you will need to install it manually.
We provide [precompiled binaries](https://github.com/latex-lsp/texlab/releases)
for Windows, Linux and macOS.
Alternatively, you can build TexLab from source or install it using your package manager.
For a list of supported package managers, you can take a look at [Repology](https://repology.org/project/texlab/versions):

[![Packaging status](https://repology.org/badge/vertical-allrepos/texlab.svg)](https://repology.org/project/texlab/versions)

### Requirements

A [TeX distribution](https://www.latex-project.org/get/#tex-distributions) is _not_ strictly required
to use the server but TexLab cannot compile your documents without one.
TexLab supports compiling using [Tectonic](https://tectonic-typesetting.github.io/).
For an example configuration, please see [here](docs/tectonic.md).

On Windows, you may need to install [Microsoft Visual C++ Redistributable for Visual Studio 2015](https://www.microsoft.com/en-US/download/details.aspx?id=48145).

### Building from Source

You will need to install the following dependencies to compile the server:

- A recent, stable version of [Rust](https://rustup.rs/)

Then run the following command in the project folder:

```shell
cargo build --release
```

Alternatively, you can install `texlab` from [crates.io](https://crates.io/crates/texlab) and run

```shell
cargo install texlab
```

## Usage

After installing an editor extension, you can simply start editing LaTeX files. All editing features work out-of-the-box over all files in the currently opened workspace.
There is no need for magic comments like `%!TEX root`
and TexLab should figure out the dependencies of a file on its own.
Note that you may need to set the `texlab.rootDirectory` option for some multi-folder projects.

TexLab features a variety of [options](docs/options.md) which can be used to configure features like building or [forward search](docs/previewing.md).

## Development

You can create a debug build by building the server without the `--release` flag.
The resulting build can be used with the [Visual Studio Code extension](https://github.com/latex-lsp/texlab-vscode)
by adding the absolute path of the `target/debug` folder to your `PATH` environment variable.

TexLab has an extensive test suite of unit and integration tests. You can run them by executing

```shell
cargo test
```

in the project folder.

For a list of custom messages, please see [here](docs/custom_messages.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
