package texlab.rename

import org.eclipse.lsp4j.WorkspaceEdit

interface Renamer {
    fun rename(request: RenameRequest): WorkspaceEdit?
}
