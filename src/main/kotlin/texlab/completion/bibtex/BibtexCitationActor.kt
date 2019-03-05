package texlab.completion.bibtex

import com.overzealous.remark.Remark
import de.undercouch.citeproc.CSL
import de.undercouch.citeproc.bibtex.BibTeXConverter
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.actor
import org.jbibtex.ParseException
import kotlin.coroutines.CoroutineContext

@Suppress("BlockingMethodInNonBlockingContext")
@ObsoleteCoroutinesApi
object BibtexCitationActor : CoroutineScope {
    override val coroutineContext: CoroutineContext = Dispatchers.IO + Job()

    private val actor = actor<BibtexCitationMessage> {
        runBlocking {
            val provider = BibtexCitationDataProvider()
            val citeproc = CSL(provider, "apa").apply { setOutputFormat("html") }
            for (message in channel) {
                val citation = try {
                    val database = BibTeXConverter().loadDatabase(message.entry.byteInputStream())
                    provider.update(database)
                    provider.registerCitationItems(citeproc)
                    val html = citeproc.makeBibliography().makeString()
                    Remark().convert(html)
                } catch (e: ParseException) {
                    null
                }
                message.response.complete(citation)
            }
        }
    }

    suspend fun cite(entry: String): String? {
        val response = CompletableDeferred<String?>()
        val message = BibtexCitationMessage(entry, response)
        actor.send(message)
        return message.response.await()
    }
}
