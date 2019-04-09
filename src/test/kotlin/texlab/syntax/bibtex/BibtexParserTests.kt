package texlab.syntax.bibtex

import io.kotlintest.data.forall
import io.kotlintest.shouldBe
import io.kotlintest.specs.StringSpec
import io.kotlintest.tables.row

class BibtexParserTests : StringSpec({
    "parse and print should be inverses of each other" {
        forall(row(1), row(2), row(3), row(4)) { index ->
            val text = BibtexParserTests::class.java.getResourceAsStream("Example$index.txt")
                    .readBytes()
                    .toString(Charsets.UTF_8)

            val tree1 = BibtexParser.parse(text)
            val tree2 = BibtexParser.parse(BibtexPrinter.print(tree1))
            tree1.shouldBe(tree2)
        }
    }

    "it should be able to parse an empty document" {
        val tree = BibtexParser.parse("")
        tree.shouldBe(BibtexDocumentSyntax(emptyList()))
    }
})
