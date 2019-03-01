package texlab

import kotlinx.coroutines.*
import kotlinx.coroutines.channels.actor
import kotlin.coroutines.CoroutineContext

@ObsoleteCoroutinesApi
class WorkspaceActor : CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.Default + Job()

    private val actor = actor<WorkspaceAction> {
        var documents = listOf<Document>()
        for (message in channel) {
            val workspace = Workspace(documents)
            when (message) {
                is WorkspaceAction.Get -> {
                    message.response.complete(workspace)
                }
                is WorkspaceAction.Put -> {
                    val document = message.updater(workspace)
                    documents = documents.filterNot { it.uri == document.uri }
                            .plus(document)
                }
            }
        }
    }

    suspend fun get(): Workspace {
        val response = CompletableDeferred<Workspace>()
        actor.send(WorkspaceAction.Get(response))
        return response.await()
    }

    suspend fun <T> withWorkspace(action: suspend (Workspace) -> T): T {
        return action(get())
    }

    suspend fun put(updater: suspend (Workspace) -> Document) {
        actor.send(WorkspaceAction.Put(updater))
    }
}
