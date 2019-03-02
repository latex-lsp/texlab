package texlab.diagnostics

import org.eclipse.lsp4j.Diagnostic
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import texlab.BibtexDocument
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.*

object BibtexEntryDiagnosticsProvider : FeatureProvider<Unit, Diagnostic> {
    override suspend fun get(request: FeatureRequest<Unit>): List<Diagnostic> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        val diagnostics = mutableListOf<Diagnostic>()
        fun add(code: ErrorCode, position: Position) {
            val range = Range(position, position)
            diagnostics.add(DiagnosticFactory.create(code, range))
        }

        for (entry in request.document.tree.root.children.filterIsInstance<BibtexEntrySyntax>()) {
            if (entry.left == null) {
                add(ErrorCode.BIBTEX_MISSING_BEGIN_BRACE, entry.type.end)
                continue
            }

            if (entry.name == null) {
                add(ErrorCode.BIBTEX_MISSING_ENTRY_NAME, entry.left.end)
                continue
            }

            if (entry.comma == null) {
                add(ErrorCode.BIBTEX_MISSING_COMMA, entry.name.end)
                continue
            }

            for (i in 0 until entry.fields.size) {
                val field = entry.fields[i]
                if (field.assign == null) {
                    add(ErrorCode.BIBTEX_MISSING_ASSIGN, field.name.end)
                    continue
                }

                if (field.content == null) {
                    add(ErrorCode.BIBTEX_MISSING_CONTENT, field.assign.end)
                    continue
                }

                diagnostics.addAll(getDiagnostics(field.content))

                if (i != entry.fields.size - 1 && field.comma == null) {
                    add(ErrorCode.BIBTEX_MISSING_COMMA, field.content.end)
                    continue
                }
            }

            if (entry.right == null) {
                val position = entry.fields.lastOrNull()?.end ?: entry.comma.end
                add(ErrorCode.BIBTEX_MISSING_END_BRACE, position)
                continue
            }
        }

        return diagnostics
    }

    private fun getDiagnostics(content: BibtexContentSyntax): List<Diagnostic> {
        val errors = mutableListOf<Diagnostic>()
        fun visit(node: BibtexContentSyntax) {
            when (node) {
                is BibtexQuotedContentSyntax -> {
                    node.children.forEach { visit(it) }
                    if (node.right == null) {
                        val range = Range(node.end, node.end)
                        errors.add(DiagnosticFactory.create(ErrorCode.BIBTEX_MISSING_QUOTE, range))
                    }
                }
                is BibtexBracedContentSyntax -> {
                    node.children.forEach { visit(it) }
                    if (node.right == null) {
                        val range = Range(node.end, node.end)
                        errors.add(DiagnosticFactory.create(ErrorCode.BIBTEX_MISSING_END_BRACE, range))
                    }
                }
                is BibtexConcatSyntax -> {
                    visit(node.left)
                    if (node.right == null) {
                        val range = Range(node.end, node.end)
                        errors.add(DiagnosticFactory.create(ErrorCode.BIBTEX_MISSING_CONTENT, range))
                    }
                }
            }
        }
        visit(content)
        return errors
    }
}
