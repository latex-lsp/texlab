package texlab

import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range

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
