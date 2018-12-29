package texlab.syntax.bibtex

import org.eclipse.lsp4j.Position
import texlab.syntax.Token

data class BibtexToken(override val start: Position,
                       override val text: String,
                       val kind: BibtexTokenKind) : Token() {
    constructor(line: Int, character: Int, text: String, kind: BibtexTokenKind)
            : this(Position(line, character), text, kind)
}

