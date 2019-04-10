package texlab.diagnostics

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.OldWorkspaceBuilder

class BibtexEntryDiagnosticsProviderTests {
    @Test
    fun `it should raise an error if the opening brace is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_BEGIN_BRACE,
                Range(Position(0, 8), Position(0, 8)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if the entry key is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_ENTRY_NAME,
                Range(Position(0, 9), Position(0, 9)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if the comma after entry name is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_COMMA,
                Range(Position(0, 12), Position(0, 12)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if the closing brace is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo,")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_END_BRACE,
                Range(Position(0, 13), Position(0, 13)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if '=' is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar}")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_ASSIGN,
                Range(Position(0, 17), Position(0, 17)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if a field has no content`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo,\nbar = }")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_CONTENT,
                Range(Position(1, 5), Position(1, 5)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if two fields are not separated by a comma`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo,\nfoo = bar\nbaz = qux}")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_COMMA,
                Range(Position(1, 9), Position(1, 9)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if a quote is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar =\n\"}")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_QUOTE,
                Range(Position(1, 1), Position(1, 1)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should raise an error if a closing brace is missing`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = \n{")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected1 = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_END_BRACE,
                Range(Position(1, 1), Position(1, 1)))

        val expected2 = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_END_BRACE,
                Range(Position(1, 1), Position(1, 1)))

        assertEquals(2, diagnostics.size)
        assertEquals(expected1, diagnostics[0])
        assertEquals(expected2, diagnostics[1])
    }

    @Test
    fun `it should raise an error if a concat operation has no right side`() = runBlocking {
        val diagnostics = OldWorkspaceBuilder()
                .document("foo.bib", "@article{foo, bar = baz \n# }")
                .diagnostics("foo.bib")
                .let { BibtexEntryDiagnosticsProvider.get(it) }

        val expected = DiagnosticFactory.create(
                ErrorCode.BIBTEX_MISSING_CONTENT,
                Range(Position(1, 1), Position(1, 1)))

        assertEquals(1, diagnostics.size)
        assertEquals(expected, diagnostics[0])
    }

    @Test
    fun `it should not process LaTeX documents`() = runBlocking<Unit> {
        OldWorkspaceBuilder()
                .document("foo.tex", "@article")
                .diagnostics("foo.tex")
                .let { BibtexEntryDiagnosticsProvider.get(it) }
                .also { assertTrue(it.isEmpty()) }
    }
}
