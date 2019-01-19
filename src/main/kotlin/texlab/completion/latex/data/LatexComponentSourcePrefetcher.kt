package texlab.completion.latex.data

import texlab.Workspace
import java.util.*
import kotlin.concurrent.timerTask

object LatexComponentSourcePrefetcher {
    fun start(workspace: Workspace, database: LatexComponentSource, period: Long = 1000): Timer {
        val task = timerTask {
            for (document in workspace.documents) {
                val relatedDocuments = workspace.relatedDocuments(document.uri)
                database.getRelatedComponents(relatedDocuments)
            }

        }
        return Timer().apply { scheduleAtFixedRate(task, 0, period) }
    }
}
