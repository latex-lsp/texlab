package texlab.diagnostics

import kotlinx.coroutines.runBlocking
import org.eclipse.lsp4j.Diagnostic
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import java.io.File

class ManualDiagnosticsProviderTests {
    @Test
    fun `it should use the mutable property for requests`() = runBlocking {
        val provider = ManualDiagnosticsProvider()
        val diagnostic1 = Diagnostic().apply { code = "1" }
        val diagnostic2 = Diagnostic().apply { code = "2" }
        val uri = File("foo.tex").toURI()
        provider.diagnosticsByUri = mapOf(uri to listOf(diagnostic1, diagnostic2))

        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .diagnostics("foo.tex")

        val diagnostics = provider.get(request)
        assertArrayEquals(arrayOf(diagnostic1, diagnostic2), diagnostics.toTypedArray())
    }

    @Test
    fun `it should return an empty list for unknown documents`() = runBlocking {
        val provider = ManualDiagnosticsProvider()
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .diagnostics("foo.tex")

        assertTrue(provider.get(request).isEmpty())
    }
}
