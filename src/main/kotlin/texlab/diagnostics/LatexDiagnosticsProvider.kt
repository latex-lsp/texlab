package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import texlab.LatexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import java.net.URI

class LatexDiagnosticsProvider : FeatureProvider<Unit, List<Diagnostic>> {
    private val diagnosticsByUri = mutableMapOf<URI, List<Diagnostic>>()

    override suspend fun get(request: FeatureRequest<Unit>): List<Diagnostic> {
        if (request.document !is LatexDocument) {
            return emptyList()
        }

        return diagnosticsByUri[request.uri].orEmpty()
    }

    suspend fun update(uri: URI, text: String) {
        diagnosticsByUri[uri] = LatexLinter.lint(text)
    }
}
