package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic

class AggregateDiagnosticsProvider(private vararg val providers: DiagnosticsProvider) : DiagnosticsProvider {
    override fun getDiagnostics(request: DiagnosticsRequest): List<Diagnostic> {
        return providers.flatMap { it.getDiagnostics(request) }
    }
}
