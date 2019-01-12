package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import java.net.URI

class ManualDiagnosticsProvider : DiagnosticsProvider {
    var diagnosticsByUri: Map<URI, List<Diagnostic>> = mapOf()

    override fun getDiagnostics(request: DiagnosticsRequest): List<Diagnostic> {
        return diagnosticsByUri[request.uri].orEmpty()
    }
}
