[![Build Status](https://dev.azure.com/latex-lsp/texlab/_apis/build/status/latex-lsp.texlab?branchName=master)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)
[![Coverage](https://img.shields.io/azure-devops/coverage/latex-lsp/texlab/8.svg?logo=azuredevops)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)
[![Dependabot](https://api.dependabot.com/badges/status?host=github&repo=latex-lsp/texlab)](https://dependabot.com)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
We provide an [extension](https://github.com/latex-lsp/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

Learn more about the project on our [website](https://texlab.netlify.com).

## Getting Started

See the [installation chapter](https://texlab.netlify.com/docs) from our docs.

## Building from Source

You will need to install the following dependencies to compile the server:

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- [Python 3](https://www.python.org/)

Then run the following commands in the project folder:

```shell
python setup.py
cargo build --release
```

## Development

You can create a debug build by building the server without the `--release` flag.
The resulting build can be used with the [Visual Studio Code extension](https://github.com/latex-lsp/texlab-vscode)
by creating a symbolic link:

| Platform     | Symlink                                                             |
| ------------ | ------------------------------------------------------------------- |
| Windows      | `texlab/target/debug/texlab.exe -> texlab-vscode/server/texlab.exe` |
| Linux, macOS | `texlab/target/debug/texlab -> texlab-vscode/server/texlab`         |

TexLab has an extensive test suite of unit and integration tests. You can run them by executing

```shell
cargo test --all
```

in the project folder.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
