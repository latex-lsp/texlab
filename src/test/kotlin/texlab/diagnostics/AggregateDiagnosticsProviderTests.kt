package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateDiagnosticsProviderTests {
    private fun createProvider(vararg diagnostics: Diagnostic): DiagnosticsProvider {
        return object : DiagnosticsProvider {
            override fun getDiagnostics(request: DiagnosticsRequest): List<Diagnostic> {
                return diagnostics.toList()
            }
        }
    }

    @Test
    fun `it should merge the diagnostics`() {
        val diagnostic1 = Diagnostic().apply { code = "1" }
        val diagnostic2 = Diagnostic().apply { code = "2" }
        val diagnostic3 = Diagnostic().apply { code = "3" }
        val provider1 = createProvider(diagnostic1, diagnostic2)
        val provider2 = createProvider(diagnostic3)
        val aggregateProvider = AggregateDiagnosticsProvider(provider1, provider2)
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .diagnostics("foo.tex")

        val expected = arrayOf(diagnostic1, diagnostic2, diagnostic3)
        assertArrayEquals(expected, aggregateProvider.getDiagnostics(request).toTypedArray())
    }
}
