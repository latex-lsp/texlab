package texlab

import org.eclipse.lsp4j.Position
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

class CharStreamTests {

    @Test
    fun `it should update the position when seeking`() {
        val stream = CharStream("foo\nbar-baz")
        val position = Position(1, 2)
        stream.seek(position)
        assertEquals(position, stream.position)
    }

    @Test
    fun `it should update the position when advancing`() {
        val stream = CharStream("a\nb")
        stream.next()
        assertEquals(Position(0, 1), stream.position)
        stream.next()
        assertEquals(Position(1, 0), stream.position)
        stream.next()
        assertEquals(Position(1, 1), stream.position)
    }

    @Test
    fun `it should not change the position when peeking`() {
        val stream = CharStream("a\nb")
        stream.peek()
        assertEquals(Position(0, 0), stream.position)
    }

    @Test
    fun `it should return the character when peeking`() {
        val stream = CharStream("abc")
        assertEquals('a', stream.peek(0))
        assertEquals('b', stream.peek(1))
        assertEquals('c', stream.peek(2))
    }

    @Test
    fun `it should be able to skip the rest of the line`() {
        val stream = CharStream("foo\nbar")
        stream.skipRestOfLine()
        assertEquals(Position(1, 0), stream.position)
    }
}
