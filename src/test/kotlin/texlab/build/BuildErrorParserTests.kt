package texlab.build

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import java.io.File
import java.net.URI

class BuildErrorParserTests {
    private val parent: URI = File("parent.tex").toURI()

    private val child: URI = File("child.tex").toURI()

    private fun verify(name: String, expected: Array<BuildError>) {
        javaClass.getResourceAsStream(name)
                .readBytes()
                .toString(Charsets.UTF_8)
                .let { BuildErrorParser.parse(parent, it) }
                .toTypedArray()
                .also { assertArrayEquals(expected, it) }
    }

    @Test
    fun `it should parse bad boxes`() {
        val errors = arrayOf(
                BuildError(parent, BuildErrorKind.WARNING,
                        "Overfull \\hbox (200.00162pt too wide) in paragraph at lines 8--9", 7),
                BuildError(parent, BuildErrorKind.WARNING,
                        "Overfull \\vbox (3.19998pt too high) detected at line 23", 22))
        verify("bad-box.log", errors)
    }

    @Test
    fun `it should find errors in related documents`() {
        val errors = arrayOf(BuildError(child, BuildErrorKind.ERROR, "Undefined control sequence.", 0))
        verify("child-error.log", errors)
    }

    @Test
    fun `it should parse citation warnings`() {
        val errors = arrayOf(
                BuildError(parent, BuildErrorKind.WARNING,
                        "Citation `foo' on page 1 undefined on input line 6.", null),
                BuildError(parent, BuildErrorKind.WARNING,
                        "There were undefined references.", null))
        verify("citation-warning.log", errors)
    }

    @Test
    fun `it should parse package errors`() {
        val errors = arrayOf(
                BuildError(parent, BuildErrorKind.ERROR,
                        "Package babel Error: Unknown option `foo'. Either you misspelled it or " +
                                "the language definition file foo.ldf was not found.", null),
                BuildError(parent, BuildErrorKind.ERROR,
                        "Package babel Error: You haven't specified a language option.", null))
        verify("package-error.log", errors)
    }

    @Test
    fun `it should parse package warnings`() {
        val errors = arrayOf(
                BuildError(parent, BuildErrorKind.WARNING,
                        "'babel/polyglossia' detected but 'csquotes' missing. Loading 'csquotes' recommended.", null),
                BuildError(parent, BuildErrorKind.WARNING,
                        "There were undefined references.", null),
                BuildError(parent, BuildErrorKind.WARNING,
                        "Please (re)run Biber on the file: parent and rerun LaTeX afterwards.", null))
        verify("package-warning.log", errors)
    }

    @Test
    fun `it should find TeX errors`() {
        val errors = arrayOf(
                BuildError(parent, BuildErrorKind.ERROR, "Undefined control sequence.", 6),
                BuildError(parent, BuildErrorKind.ERROR, "Missing \$ inserted.", 7),
                BuildError(parent, BuildErrorKind.ERROR, "Undefined control sequence.", 8),
                BuildError(parent, BuildErrorKind.ERROR, "Missing { inserted.", 9),
                BuildError(parent, BuildErrorKind.ERROR, "Missing \$ inserted.", 9),
                BuildError(parent, BuildErrorKind.ERROR, "Missing } inserted.", 9))
        verify("tex-error.log", errors)
    }
}
