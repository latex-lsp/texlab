package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import java.net.URI

class ManualDiagnosticsProvider : FeatureProvider<Unit, Diagnostic> {
    var diagnosticsByUri: Map<URI, List<Diagnostic>> = mapOf()

    override suspend fun get(request: FeatureRequest<Unit>): List<Diagnostic> {
        return diagnosticsByUri[request.uri].orEmpty()
    }
}
