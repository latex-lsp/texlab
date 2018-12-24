package texlab.rename

import org.eclipse.lsp4j.WorkspaceEdit

class AggregateRenamer(private vararg val renamers: Renamer) : Renamer {
    override fun rename(request: RenameRequest): WorkspaceEdit? {
        for (renamer in renamers) {
            val edit = renamer.rename(request)
            if (edit != null) {
                return edit
            }
        }
        return null
    }
}
