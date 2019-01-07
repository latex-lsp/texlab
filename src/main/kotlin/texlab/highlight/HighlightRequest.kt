package texlab.highlight

import org.eclipse.lsp4j.Position
import texlab.Document

data class HighlightRequest(val document: Document, val position: Position)
