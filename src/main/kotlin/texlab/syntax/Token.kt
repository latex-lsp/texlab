package texlab.syntax

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range

abstract class Token {
    abstract val start: Position

    val line: Int
        get() = start.line

    val character: Int
        get() = start.character

    val end: Position
        get() = Position(start.line, start.character + text.length)

    val range: Range
        get () = Range(start, end)

    abstract val text: String
}
