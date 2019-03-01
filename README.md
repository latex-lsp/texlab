[![Build Status](https://travis-ci.com/efoerster/texlab.svg?token=ecjo579NH2soHXd6GykN&branch=master)](https://travis-ci.com/efoerster/texlab) [![codecov](https://codecov.io/gh/efoerster/texlab/branch/master/graph/badge.svg?token=485LvHBRXW)](https://codecov.io/gh/efoerster/texlab)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.
We provide an [extension](https://github.com/efoerster/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

Learn more about the project on our [website](https://texlab.netlify.com).

## Getting Started

This project uses [Gradle](https://gradle.org/).
To compile the server and run the tests execute the following command in the project directory:

```shell
./gradlew build
```

To use the local build with the [extension](https://github.com/efoerster/texlab-vscode), we recommend creating a symbolic link:

```shell
mkdir ../texlab-vscode/server
ln -s ../../texlab/build/libs/texlab.jar ./../texlab-vscode/server/texlab.jar
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
