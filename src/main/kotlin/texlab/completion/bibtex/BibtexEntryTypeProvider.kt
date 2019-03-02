package texlab.completion.bibtex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.BibtexDocument
import texlab.completion.CompletionItemFactory
import texlab.contains
import texlab.provider.FeatureProvider
import texlab.provider.FeatureRequest
import texlab.syntax.bibtex.BibtexDeclarationSyntax

object BibtexEntryTypeProvider : FeatureProvider<CompletionParams, CompletionItem> {
    private val ENTRY_TYPES = arrayOf("preamble", "string", "article", "book", "mvbook", "inbook", "bookinbook",
            "suppbook", "booklet", "collection", "mvcollection", "incollection", "suppcollection", "manual", "misc",
            "online", "patent", "periodical", "suppperiodical", "proceedings", "mvproceedings", "inproceedings",
            "reference", "mvreference", "inreference", "report", "set", "thesis", "unpublished", "xdata",
            "conference", "electronic", "mastersthesis", "phdthesis", "techreport", "www", "artwork", "audio",
            "commentary", "image", "jurisdiction", "legislation", "legal", "letter", "movie", "music", "performance",
            "review", "software", "standard", "video")

    private val items = ENTRY_TYPES.map { CompletionItemFactory.createEntryType(it) }

    override suspend fun get(request: FeatureRequest<CompletionParams>): List<CompletionItem> {
        if (request.document !is BibtexDocument) {
            return emptyList()
        }

        for (node in request.document.tree.root.children.filterIsInstance<BibtexDeclarationSyntax>()) {
            if (node.type.range.contains(request.params.position)) {
                return items
            }
        }

        return emptyList()
    }
}
