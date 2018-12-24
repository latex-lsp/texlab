package texlab

import org.eclipse.lsp4j.DocumentLink
import org.eclipse.lsp4j.DocumentSymbol
import org.eclipse.lsp4j.FoldingRange
import org.eclipse.lsp4j.FoldingRangeKind
import texlab.syntax.latex.LatexEnvironmentSymbolFinder
import texlab.syntax.latex.LatexSection
import texlab.syntax.latex.LatexSyntaxTree
import java.net.URI

class LatexDocument(uri: URI) : Document(uri) {

    var tree: LatexSyntaxTree = LatexSyntaxTree(text)

    override fun analyze() {
        tree = LatexSyntaxTree(text)
    }

    override fun documentSymbol(workspace: Workspace): List<DocumentSymbol> {
        return LatexEnvironmentSymbolFinder.find(tree)
    }

    override fun documentLink(workspace: Workspace): List<DocumentLink> {
        return tree.includes.mapNotNull {
            val range = it.command.args[0].children[0].range
            val target = workspace.resolve(uri, it.path) ?: return@mapNotNull null
            DocumentLink(range, target.uri.toString())
        }
    }

    override fun foldingRange(): List<FoldingRange> {
        val foldings = mutableListOf<FoldingRange>()
        for (environment in tree.environments) {
            val folding = FoldingRange().apply {
                startLine = environment.begin.end.line
                startCharacter = environment.begin.end.character
                endLine = environment.end.start.line
                endCharacter = environment.end.start.character
                kind = FoldingRangeKind.Region
            }
            foldings.add(folding)
        }

        val sections = tree.sections
        for (i in 0 until sections.size) {
            val current = sections[i]
            var next: LatexSection? = null
            for (j in i + 1 until sections.size) {
                next = sections[j]
                if (current.level >= sections[j].level) {
                    break
                }
            }

            if (next != null) {
                val folding = FoldingRange().apply {
                    startLine = current.command.range.end.line
                    startCharacter = current.command.range.end.character
                    endLine = next.command.range.start.line - 1
                    endCharacter = 0
                    kind = FoldingRangeKind.Region
                }
                foldings.add(folding)
            }
        }
        return foldings
    }
}
