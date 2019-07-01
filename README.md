[![Build Status](https://dev.azure.com/latex-lsp/texlab/_apis/build/status/latex-lsp.texlab?branchName=master)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)
[![Coverage](https://img.shields.io/azure-devops/coverage/latex-lsp/texlab/8.svg?logo=azuredevops)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
We provide an [extension](https://github.com/latex-lsp/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

Learn more about the project on our [website](https://texlab.netlify.com).

## Installation

If you want to use the server with [Visual Studio Code](https://code.visualstudio.com), then you can simply install
our extension from the [marketplace](https://marketplace.visualstudio.com/items?itemName=efoerster.texlab).

For other [tools](https://microsoft.github.io/language-server-protocol/implementors/tools/)
implementing the [Language Server Protocol](https://microsoft.github.io/language-server-protocol),
we provide [precompiled binaries for Windows, Linux and macOS](https://github.com/latex-lsp/texlab/releases).
You can place them on any directory that is in your `PATH`, for example `/usr/local/bin`
on Linux and macOS. On Windows, you will need to install
[Microsoft Visual C++ Redistributable for Visual Studio 2015](https://aka.ms/vs/16/release/vc_redist.x64.exe) to run the server.

## Building

You will need to install the following dependencies to compile the server:

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)

Then run the following commands in the project folder:

```shell
cd citeproc
npm install
npm run dist
cd ..
cargo build --release
```

## Development

You can create a debug build by building the server without the `--release` flag.
The resulting build can be used with the [Visual Studio Code extension](https://github.com/latex-lsp/texlab-vscode)
by creating a symbolic link:

| Platform    | Symlink                                                                            |
| ----------- | ---------------------------------------------------------------------------------- |
| Windows x64 | `texlab/target/debug/texlab.exe -> texlab-vscode/server/texlab-x86_64-windows.exe` |
| macOS x64   | `texlab/target/debug/texlab -> texlab-vscode/server/texlab-x86_64-darwin`          |
| Linux x64   | `texlab/target/debug/texlab -> texlab-vscode/server/texlab-x86_64-linux`           |

TexLab has an extensive test suite of unit and integration tests. You can run them by executing

```shell
cargo test --all
```

in the project folder.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
