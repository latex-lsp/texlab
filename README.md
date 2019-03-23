[![Build Status](https://dev.azure.com/latex-lsp/texlab/_apis/build/status/latex-lsp.texlab?branchName=master)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)
[![codecov](https://codecov.io/gh/latex-lsp/texlab/branch/master/graph/badge.svg)](https://codecov.io/gh/latex-lsp/texlab)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
We provide an [extension](https://github.com/latex-lsp/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

Learn more about the project on our [website](https://texlab.netlify.com).

## Getting Started

This project uses [Yarn](https://yarnpkg.com/). See `package.json` for all available scripts.
In order to use the local build with the [extension](https://github.com/latex-lsp/texlab-vscode), create a symbolic link:

```shell
yarn link
cd ../texlab-vscode
yarn link texlab
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
