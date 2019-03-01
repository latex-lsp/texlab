package texlab.completion.bibtex

import com.overzealous.remark.Remark
import de.undercouch.citeproc.CSL
import de.undercouch.citeproc.bibtex.BibTeXConverter
import org.jbibtex.ParseException

object BibtexCitationGenerator {
    private val provider = BibtexCitationDataProvider()
    private val citeproc = CSL(provider, "apa").apply { setOutputFormat("html") }

    fun cite(entry: String): String? {
        synchronized(citeproc) {
            return try {
                val database = BibTeXConverter().loadDatabase(entry.byteInputStream())
                provider.update(database)
                provider.registerCitationItems(citeproc)
                val html = citeproc.makeBibliography().makeString()
                Remark().convert(html)
            } catch (e: ParseException) {
                null
            }
        }
    }
}
