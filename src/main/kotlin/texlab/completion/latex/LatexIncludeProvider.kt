package texlab.completion.latex

import texlab.LatexDocument
import texlab.Workspace

class LatexIncludeProvider(workspace: Workspace) :
        DocumentProvider<LatexDocument>(workspace, LatexDocument::class.java, false) {
    override val commandNames: List<String> = listOf("\\include")
}
