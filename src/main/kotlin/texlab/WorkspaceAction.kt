package texlab

import kotlinx.coroutines.CompletableDeferred

sealed class WorkspaceAction {
    class Get(val response: CompletableDeferred<Workspace>) : WorkspaceAction()

    class Put(val updater: (Workspace) -> Document) : WorkspaceAction()
}
