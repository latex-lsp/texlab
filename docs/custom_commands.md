# Custom Commands

The server provides the following commands through the `workspace/executeCommand` request:

## texlab.cleanAuxiliary

Removes the auxiliary files produced by compiling the specified LaTeX document.
At the moment, this command simply calls `latexmk -c` with the currently configured output directory.

Parameters:

- `document`: `TextDocumentIdentifier` (_Required_)

## texlab.cleanArtifacts

Removes the auxiliary files and the artifacts produced by compiling the specified LaTeX document.
At the moment, this command simply calls `latexmk -C` with the currently configured output directory.

Parameters:

- `document`: `TextDocumentIdentifier` (_Required_)
