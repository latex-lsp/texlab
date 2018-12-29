package texlab.syntax

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range

abstract class SyntaxNode {
    abstract val range: Range

    val start: Position
        get() = range.start

    val end: Position
        get() = range.end
}
