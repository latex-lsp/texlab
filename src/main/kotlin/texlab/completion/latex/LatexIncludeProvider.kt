package texlab.completion.latex

import texlab.LatexDocument

class LatexIncludeProvider : DocumentProvider<LatexDocument>(LatexDocument::class.java, false) {
    override val commandNames: List<String> = listOf("\\include")
}
