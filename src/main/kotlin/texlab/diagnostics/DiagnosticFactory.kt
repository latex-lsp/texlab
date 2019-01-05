package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import org.eclipse.lsp4j.DiagnosticSeverity
import org.eclipse.lsp4j.Range

object DiagnosticFactory {
    fun create(code: ErrorCode, range: Range): Diagnostic {
        val message = when (code) {
            ErrorCode.BIBTEX_MISSING_BEGIN_BRACE ->
                "Expecting a curly bracket: \"{\""
            ErrorCode.BIBTEX_MISSING_ENTRY_NAME ->
                "Expecting an entry name"
            ErrorCode.BIBTEX_MISSING_COMMA ->
                "Expecting a comma: \",\""
            ErrorCode.BIBTEX_MISSING_END_BRACE ->
                "Expecting a curly bracket: \"}\""
            ErrorCode.BIBTEX_MISSING_ASSIGN ->
                "Expecting an equals sign: \"=\""
            ErrorCode.BIBTEX_MISSING_CONTENT ->
                "Expecting content"
            ErrorCode.BIBTEX_MISSING_QUOTE ->
                "Expecting a quote: '\"'"
        }

        val severity = when (code) {
            ErrorCode.BIBTEX_MISSING_BEGIN_BRACE ->
                DiagnosticSeverity.Error
            ErrorCode.BIBTEX_MISSING_ENTRY_NAME ->
                DiagnosticSeverity.Error
            ErrorCode.BIBTEX_MISSING_COMMA ->
                DiagnosticSeverity.Error
            ErrorCode.BIBTEX_MISSING_END_BRACE ->
                DiagnosticSeverity.Error
            ErrorCode.BIBTEX_MISSING_ASSIGN ->
                DiagnosticSeverity.Error
            ErrorCode.BIBTEX_MISSING_CONTENT ->
                DiagnosticSeverity.Error
            ErrorCode.BIBTEX_MISSING_QUOTE ->
                DiagnosticSeverity.Error
        }

        return Diagnostic(range, message, severity, "texlab")
    }
}
