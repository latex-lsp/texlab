[![Build Status](https://travis-ci.com/efoerster/texlab.svg?token=ecjo579NH2soHXd6GykN&branch=master)](https://travis-ci.com/efoerster/texlab)

# TexLab

A cross-platform implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol)
providing rich cross-editing support for the [LaTeX](https://www.latex-project.org/) typesetting system.

The server is designed to be independent of the editor.
We provide a [reference implementation](https://github.com/efoerster/texlab-vscode) for [Visual Studio Code](https://code.visualstudio.com).

## Getting Started

This project uses [Gradle](https://gradle.org/).
To compile the server and run the tests execute the following command in the project directory:

```shell
$ ./gradlew build
```

To use the local build with the [extension](https://github.com/efoerster/texlab-vscode), we recommend creating a symbolic link:

```shell
$ ln -s ./../texlab-vscode/server/texlab.jar ./build/libs/texlab.jar
```

## Custom Messages

### Build Request

The build request is sent from the client to the server to compile a given LaTeX document.

_Request_:

- method: 'textDocument/build'
- params: `BuildTextDocumentParams` defined as follows:

```typescript
interface BuildTextDocumentParams {
  textDocument: TextDocumentIdentifier;
}
```

_Response_:

- result: `BuildStatus` defined as follows:

```typescript
enum BuildStatus {
  Success = 0,
  Error = 1,
  Failure = 2
}
```

### Forward Search Request

The forward search request is sent from the client to the server when the user requests a forward search via SyncTeX.

_Request_:

- method: 'textDocument/forwardSearch'
- params: [`TextDocumentPositionParams`](https://microsoft.github.io/language-server-protocol/specification#textdocumentpositionparams)

_Response_:

- result: `ForwardSearchStatus` defined as follows:

```typescript
enum ForwardSearchStatus {
  Success = 0,
  Error = 1,
  Unconfigured = 2
}
```

### Status Notification

The status notification is sent from the server to the client to inform the client about the status of the server.

_Notification_:

- method: 'window/setStatus'
- params: `StatusParams` defined as follows:

```typescript
interface StatusParams {
  status: ServerStatus;
  uri?: string;
}

enum ServerStatus {
  Idle = 0,
  Building = 1,
  Indexing = 2
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.
