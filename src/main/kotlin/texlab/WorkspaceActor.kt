package texlab

import kotlinx.coroutines.*
import kotlinx.coroutines.channels.actor
import kotlin.coroutines.CoroutineContext

@ObsoleteCoroutinesApi
class WorkspaceActor : CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.Default + Job()

    private val actor = actor<Action> {
        var documents = listOf<Document>()
        for (message in channel) {
            val workspace = Workspace(documents)
            when (message) {
                is Action.Get -> {
                    message.response.complete(workspace)
                }
                is Action.Put -> {
                    val document = message.updater(workspace)
                    documents = documents.filterNot { it.uri == document.uri }
                            .plus(document)
                }
            }
        }
    }

    suspend fun get(): Workspace {
        val response = CompletableDeferred<Workspace>()
        actor.send(Action.Get(response))
        return response.await()
    }

    suspend fun <T> withWorkspace(action: suspend (Workspace) -> T): T {
        return action(get())
    }

    fun put(updater: (Workspace) -> Document) = runBlocking {
        actor.send(Action.Put(updater))
    }

    private sealed class Action {
        class Get(val response: CompletableDeferred<Workspace>) : Action()

        class Put(val updater: (Workspace) -> Document) : Action()
    }
}
