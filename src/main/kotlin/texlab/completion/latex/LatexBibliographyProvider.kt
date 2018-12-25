package texlab.completion.latex

import texlab.BibtexDocument
import texlab.Workspace

class LatexBibliographyProvider(workspace: Workspace) :
        IncludeProvider<BibtexDocument>(workspace, BibtexDocument::class.java) {
    override val commandNames: List<String> = listOf("\\bibliography", "\\addbibresource")
}
