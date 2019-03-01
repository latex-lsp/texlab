package texlab.completion.latex

import texlab.LatexDocument

class LatexInputProvider : DocumentProvider<LatexDocument>(LatexDocument::class.java, true) {
    override val commandNames: List<String> = listOf("\\input")
}
