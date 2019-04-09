package texlab

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range

fun range(startLine: Int,
          startCharacter: Int,
          endLine: Int,
          endCharacter: Int): Range {
    val start = Position(startLine, startCharacter)
    val end = Position(endLine, endCharacter)
    return Range(start, end)
}

fun Range.contains(position: Position): Boolean {
    fun leq(left: Position, right: Position): Boolean {
        return if (left.line == right.line) {
            left.character <= right.character
        } else {
            left.line <= right.line
        }
    }
    return leq(start, position) && leq(position, end)
}

fun Range.containsExclusive(position: Position): Boolean {
    fun lt(left: Position, right: Position): Boolean {
        return if (left.line == right.line) {
            left.character < right.character
        } else {
            left.line < right.line
        }
    }
    return lt(start, position) && lt(position, end)
}