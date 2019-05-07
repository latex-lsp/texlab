[![Build Status](https://dev.azure.com/latex-lsp/texlab/_apis/build/status/latex-lsp.texlab?branchName=master)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)
[![Coverage](https://img.shields.io/azure-devops/coverage/latex-lsp/texlab/8.svg?logo=azuredevops)](https://dev.azure.com/latex-lsp/texlab/_build/latest?definitionId=8&branchName=master)
[![Maintainability](https://api.codeclimate.com/v1/badges/9ce99ec1116b43ee3fc4/maintainability)](https://codeclimate.com/github/latex-lsp/texlab/maintainability)

# TexLab

_Note: The server is currently being rewritten in Rust. You can follow the progress [here](https://github.com/latex-lsp/texlab/projects/1)._

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
We provide an [extension](https://github.com/latex-lsp/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

Learn more about the project on our [website](https://texlab.netlify.com).

## Getting Started

This project uses [Gradle](https://gradle.org/).
To compile the server and run the tests execute the following command in the project directory:

```shell
./gradlew build
```

To use the local build with the [extension](https://github.com/latex-lsp/texlab-vscode), we recommend creating a symbolic link:

```shell
mkdir ../texlab-vscode/server
ln -s ../../texlab/build/libs/texlab.jar ./../texlab-vscode/server/texlab.jar
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
