package texlab

import org.eclipse.lsp4j.Position

class CharStream(val text: String) {
    private var line: Int = 0
    private var character: Int = 0

    var index: Int = 0
        private set

    val position: Position
        get() = Position(line, character)

    val available: Boolean
        get() = index < text.length

    fun peek(lookAhead: Int = 0): Char = text[index + lookAhead]

    fun next(): Char {
        if (text[index] == '\n') {
            line++
            character = 0
        } else {
            character++
        }
        return text[index++]
    }

    fun seek(newPosition: Position) {
        while (line < newPosition.line) {
            next()
        }

        while (character < newPosition.character) {
            next()
        }
    }

    fun skipRestOfLine() {
        while (available) {
            if (peek() == '\n') {
                next()
                break
            } else {
                next()
            }
        }
    }
}
