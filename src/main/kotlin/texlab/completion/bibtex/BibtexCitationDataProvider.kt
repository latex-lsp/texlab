package texlab.completion.bibtex

import de.undercouch.citeproc.CSL
import de.undercouch.citeproc.ListItemDataProvider
import de.undercouch.citeproc.bibtex.BibTeXConverter
import org.jbibtex.BibTeXDatabase

class BibtexCitationDataProvider : ListItemDataProvider() {
    fun update(database: BibTeXDatabase) {
        items.clear()
        items.putAll(BibTeXConverter().toItemData(database))
    }

    fun registerCitationItems(citeproc: CSL) {
        citeproc.reset()
        citeproc.registerCitationItems(*ids)
    }
}
