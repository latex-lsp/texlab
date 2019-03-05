package texlab

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import java.net.URI
import java.nio.file.Files
import java.nio.file.InvalidPathException
import java.nio.file.Path
import java.nio.file.Paths
import java.util.*

data class Workspace(val documents: List<Document> = listOf()) {
    fun resolveDocument(uri: URI, relativePath: String): Document? {
        for (target in resolveLinkTargets(uri, relativePath)) {
            val child = File(target).toURI()
            val document = documents.filter { it.isFile }.firstOrNull { it.uri == child }
            if (document != null) {
                return document
            }
        }
        return null
    }

    fun resolveLinkTargets(uri: URI, relativePath: String): List<String> {
        if (uri.scheme != "file") {
            return emptyList()
        }

        val targets = mutableListOf<String>()
        val extensions = arrayOf(".tex", ".sty", ".cls", ".bib")
        return try {
            val basePath = Paths.get(File(uri).parent)
            val fullPath = basePath.resolve(relativePath)
                    .normalize()
                    .toString()
                    .replace('\\', '/')
            targets.add(fullPath)
            extensions.forEach { targets.add("$fullPath$it") }
            return targets
        } catch (e: InvalidPathException) {
            emptyList()
        }
    }

    fun relatedDocuments(uri: URI): List<Document> {
        val edges = mutableSetOf<Pair<Document, Document>>()
        documents.filterIsInstance<LatexDocument>()
                .filter { it.isFile }
                .forEach { parent ->
                    parent.tree.includes
                            .mapNotNull { resolveDocument(parent.uri, it.path) }
                            .forEach { child ->
                                edges.add(Pair(parent, child))
                                edges.add(Pair(child, parent))
                            }
                }

        val results = mutableListOf<Document>()
        val start = documents.firstOrNull { it.uri == uri } ?: return results
        val visited = mutableSetOf<Document>()
        val stack = Stack<Document>()
        stack.push(start)
        while (!stack.empty()) {
            val current = stack.pop()
            if (!visited.add(current)) {
                continue
            }

            results.add(current)
            documents.filter { edges.contains(Pair(current, it)) }
                    .forEach { stack.push(it) }
        }
        return results
    }

    fun findParent(childUri: URI): Document {
        return relatedDocuments(childUri)
                .filterIsInstance<LatexDocument>()
                .firstOrNull { it.tree.isStandalone }
                ?: documents.first { it.uri == childUri }
    }

    companion object {
        suspend fun load(file: Path): Document? {
            val extension = file.fileName.toFile().extension
            val language = getLanguageByExtension(extension) ?: return null
            return try {
                val text = withContext(Dispatchers.IO) {
                    Files.readAllBytes(file).toString(Charsets.UTF_8)
                }
                Document.create(file.toUri(), text, language)
            } catch (e: IOException) {
                // File is locked
                null
            }
        }
    }
}
