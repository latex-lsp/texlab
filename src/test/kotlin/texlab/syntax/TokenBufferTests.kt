package texlab.syntax

import io.kotlintest.matchers.boolean.shouldBeFalse
import io.kotlintest.matchers.boolean.shouldBeTrue
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import java.util.*

class TokenBufferTests : StringSpec({
    fun <T> createBuffer(vararg items: T): TokenBuffer<T> {
        val queue = ArrayDeque(items.toList())
        return TokenBuffer(object : TokenSource<T> {
            override fun next(): T? {
                return queue.poll()
            }
        })
    }

    "peek should not update the position" {
        val buffer = createBuffer(1, 2, 3)
        buffer.peek(0).shouldBe(1)
        buffer.peek(1).shouldBe(2)
        buffer.peek(2).shouldBe(3)
        buffer.available.shouldBeTrue()
    }

    "next should update the position" {
        val buffer = createBuffer(1, 2, 3)
        buffer.next().shouldBe(1)
        buffer.next().shouldBe(2)
        buffer.next().shouldBe(3)
        buffer.available.shouldBeFalse()
    }
})