package texlab.syntax

import org.eclipse.lsp4j.Range
import texlab.syntax.bibtex.BibtexDocumentSyntax
import texlab.syntax.bibtex.BibtexParser
import texlab.syntax.latex.*

sealed class SyntaxTree {
    abstract val text: String

    abstract val root: SyntaxNode

    fun extract(range: Range): String {
        val stream = CharStream(text)
        stream.seek(range.start)
        val startIndex = stream.index
        stream.seek(range.end)
        val endIndex = stream.index
        return text.substring(startIndex, endIndex)
    }
}

class BibtexSyntaxTree(override val text: String) : SyntaxTree() {
    override val root: BibtexDocumentSyntax = BibtexParser.parse(text)
}

class LatexSyntaxTree(override val text: String) : SyntaxTree() {
    override val root: LatexDocumentSyntax = LatexParser.parse(text)

    val includes: List<LatexInclude> = LatexInclude.find(root)

    val components: List<String> = includes.mapNotNull {
        when (it.command.name.text) {
            "\\usepackage" -> it.path + ".sty"
            "\\documentclass" -> it.path + ".cls"
            else -> null
        }
    }

    val environments: List<LatexEnvironment> = LatexEnvironment.find(root)

    val sections: List<LatexSection> = LatexSection.find(root)

    val labelDefinitions: List<LatexLabel> = LatexLabel.findDefinitions(root)

    val labelReferences: List<LatexLabel> = LatexLabel.findReferences(root)

    val citations: List<LatexCitation> = LatexCitation.find(root)

    val equations: List<LatexEquation> = LatexEquation.find(root)

    val inlines: List<LatexInline> = LatexInline.find(root)

    val isStandalone: Boolean = environments.any { it.beginName == "document" || it.endName == "document" }

}
