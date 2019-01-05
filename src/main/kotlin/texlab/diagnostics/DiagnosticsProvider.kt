package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic

interface DiagnosticsProvider {
    fun getDiagnostics(request: DiagnosticsRequest): List<Diagnostic>
}

