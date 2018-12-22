package texlab.syntax.latex

import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.SymbolKind

abstract class LatexEnvironmentSymbolFinder {
    companion object {
        val kind: SymbolKind = SymbolKind.EnumMember

        fun find(tree: LatexSyntaxTree): List<DocumentSymbol> {
            val symbols = mutableListOf<DocumentSymbol>()
            for (environment in tree.environments) {
                symbols.add(
                        DocumentSymbol(
                                environment.beginName,
                                kind,
                                environment.beginNameRange,
                                environment.beginNameRange))
                symbols.add(
                        DocumentSymbol(
                                environment.endName,
                                kind,
                                environment.endNameRange,
                                environment.endNameRange))
            }
            return symbols
        }
    }
}
