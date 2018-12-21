package texlab.latex

import org.eclipse.lsp4j.Position
import texlab.Token

data class LatexToken(override val start: Position,
                      override val text: String,
                      val kind: LatexTokenKind) : Token() {

    constructor(line: Int, character: Int, text: String, kind: LatexTokenKind)
            : this(Position(line, character), text, kind)
}
