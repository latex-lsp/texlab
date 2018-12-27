package texlab.build

import org.eclipse.lsp4j.TextDocumentIdentifier

data class BuildParams(var textDocument: TextDocumentIdentifier = TextDocumentIdentifier())
