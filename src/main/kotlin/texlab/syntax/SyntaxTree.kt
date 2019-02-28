package texlab.syntax

import texlab.syntax.bibtex.BibtexDocumentSyntax
import texlab.syntax.bibtex.BibtexParser
import texlab.syntax.latex.*

sealed class SyntaxTree {
    abstract val root: SyntaxNode
}

class BibtexSyntaxTree(text: String) : SyntaxTree() {
    override val root: BibtexDocumentSyntax = BibtexParser.parse(text)
}

class LatexSyntaxTree(text: String) : SyntaxTree() {
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

    val isStandalone: Boolean = environments.any { it.beginName == "document" || it.endName == "document" }
}
