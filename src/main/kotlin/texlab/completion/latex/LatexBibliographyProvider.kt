package texlab.completion.latex

import texlab.BibtexDocument

class LatexBibliographyProvider : DocumentProvider<BibtexDocument>(BibtexDocument::class.java, true) {
    override val commandNames: List<String> = listOf("\\bibliography", "\\addbibresource")
}
