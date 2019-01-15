package texlab.completion.latex

import texlab.BibtexDocument
import texlab.Workspace

class LatexBibliographyProvider(workspace: Workspace) :
        DocumentProvider<BibtexDocument>(workspace, BibtexDocument::class.java, true) {
    override val commandNames: List<String> = listOf("\\bibliography", "\\addbibresource")
}
