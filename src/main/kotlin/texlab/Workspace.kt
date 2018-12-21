package texlab

import org.eclipse.lsp4j.TextDocumentContentChangeEvent
import java.io.IOException
import java.net.URI
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.PathMatcher

class Workspace {
    private val documents = mutableListOf<Document>()
    private val matcher: PathMatcher = FileSystems.getDefault().getPathMatcher("glob:**.{tex,sty,cls,bib}")

    fun initialize(directory: Path) {
        Files.walk(directory)
                .filter { Files.isRegularFile(it) }
                .filter { matcher.matches(it) }
                .forEach {
                    val extension = it.fileName.toFile().extension
                    val language = getLanguageByExtension(extension) ?: return@forEach
                    try {
                        val text = Files.readAllBytes(it).toString(Charsets.UTF_8)
                        create(it.toUri(), text, language)
                    } catch (e: IOException) {
                        // TODO: Log this error
                    }
                }
    }

    fun create(uri: URI, text: String, language: Language) {
        var document = documents.firstOrNull { it.uri == uri }
        if (document == null) {
            document = when (language) {
                Language.LATEX ->
                    LatexDocument(uri)
                Language.BIBTEX ->
                    BibtexDocument(uri)
            }

            val change = TextDocumentContentChangeEvent(text)
            document.update(listOf(change), 0)
            documents.add(document)
        }
    }

    fun update(uri: URI, changes: List<TextDocumentContentChangeEvent>, version: Int) {
        val document = documents.firstOrNull { it.uri == uri } ?: return
        document.update(changes, version)
    }
}
