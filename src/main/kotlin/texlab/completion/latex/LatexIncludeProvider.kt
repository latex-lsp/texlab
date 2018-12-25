package texlab.completion.latex

import texlab.LatexDocument
import texlab.Workspace

class LatexIncludeProvider(workspace: Workspace) :
        IncludeProvider<LatexDocument>(workspace, LatexDocument::class.java) {
    override val commandNames: List<String> = listOf("\\include", "\\input")
}

