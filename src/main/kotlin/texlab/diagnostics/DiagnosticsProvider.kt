package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest

class DiagnosticsProvider : FeatureProvider<Unit, List<Diagnostic>> {
    val buildProvider = ManualDiagnosticsProvider()
    val latexProvider = LatexDiagnosticsProvider()

    private val provider = FeatureProvider.concat(
            buildProvider,
            BibtexEntryDiagnosticsProvider,
            latexProvider)

    override suspend fun get(request: FeatureRequest<Unit>): List<Diagnostic> {
        return provider.get(request)
    }
}
