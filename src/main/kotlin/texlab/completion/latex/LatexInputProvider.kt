package texlab.completion.latex

import texlab.LatexDocument
import texlab.Workspace

class LatexInputProvider(workspace: Workspace) :
        DocumentProvider<LatexDocument>(workspace, LatexDocument::class.java, true) {
    override val commandNames: List<String> = listOf("\\input")
}
