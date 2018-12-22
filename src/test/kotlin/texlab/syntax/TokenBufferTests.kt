package texlab.syntax

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import java.util.*

class TokenBufferTests {

    private fun <T> createBuffer(vararg items: T): TokenBuffer<T> {
        val queue = ArrayDeque<T>()
        items.forEach { queue.offer(it) }
        return TokenBuffer(object : TokenSource<T> {
            override fun next(): T? {
                return queue.poll()
            }
        })
    }

    @Test
    fun `it should return the correct item when peeking`() {
        val buffer = createBuffer(1, 2, 3)
        assertEquals(1, buffer.peek(0))
        assertEquals(2, buffer.peek(1))
        assertEquals(3, buffer.peek(2))
    }

    @Test
    fun `it should return the correct item when advancing`() {
        val buffer = createBuffer(1, 2, 3)
        assertTrue(buffer.available)
        assertEquals(1, buffer.next())
        assertEquals(2, buffer.next())
        assertEquals(3, buffer.next())
        assertFalse(buffer.available)
    }
}
