package texlab.build

import org.eclipse.lsp4j.Diagnostic
import org.eclipse.lsp4j.DiagnosticSeverity
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import java.net.URI

data class BuildError(val uri: URI,
                      val kind: BuildErrorKind,
                      val message: String,
                      val line: Int?) {
    fun toDiagnostic(): Diagnostic {
        val position = Position(line ?: 0, 0)
        val severity = when (kind) {
            BuildErrorKind.ERROR ->
                DiagnosticSeverity.Error
            BuildErrorKind.WARNING ->
                DiagnosticSeverity.Warning
        }
        val range = Range(position, position)
        return Diagnostic(range, message, severity, "texlab")
    }
}

