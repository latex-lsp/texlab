package texlab.syntax

import io.kotlintest.matchers.boolean.shouldBeFalse
import io.kotlintest.matchers.boolean.shouldBeTrue
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import org.eclipse.lsp4j.Position

class CharStreamTests : StringSpec({
    "peek should not update the position" {
        val stream = CharStream("abc")
        stream.peek(0).shouldBe('a')
        stream.peek(1).shouldBe('b')
        stream.peek(2).shouldBe('c')
        stream.position.shouldBe(Position(0, 0))
        stream.index.shouldBe(0)
    }

    "next should update the position" {
        val stream = CharStream("a\nb")
        stream.position.shouldBe(Position(0, 0))
        stream.next().shouldBe('a')
        stream.position.shouldBe(Position(0, 1))
        stream.next().shouldBe('\n')
        stream.position.shouldBe(Position(1, 0))
        stream.next().shouldBe('b')
        stream.position.shouldBe(Position(1, 1))
    }

    "seek should update the position" {
        val stream = CharStream("abc\ndef\nghi")
        stream.seek(Position(1, 2))
        stream.position.shouldBe(Position(1, 2))
    }

    "skipRestOfLine should update the position" {
        val stream = CharStream("abc\ndef\nghi")
        stream.skipRestOfLine()
        stream.position.shouldBe(Position(1, 0))
    }

    "available should behave as expected" {
        val stream = CharStream("a")
        stream.available.shouldBeTrue()
        stream.next()
        stream.available.shouldBeFalse()
    }
})

