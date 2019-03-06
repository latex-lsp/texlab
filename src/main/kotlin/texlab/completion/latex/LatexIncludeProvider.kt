package texlab.completion.latex

import org.apache.commons.io.FilenameUtils
import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax
import java.io.File
import java.io.IOException
import java.nio.file.Files
import java.nio.file.InvalidPathException
import java.nio.file.Path

object LatexIncludeProvider : LatexArgumentProvider() {
    override val commandNames: List<String> = listOf(
            "\\include", "\\input", "\\bibliography",
            "\\addbibresource", "\\includegraphics", "\\includesvg")

    override val argumentIndex: Int = 0

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        if (!request.document.isFile) {
            return emptyList()
        }

        return try {
            val directory = getCurrentDirectory(request, command)
            val entries = mutableListOf<CompletionItem>()
            for (entry in Files.walk(directory, 1)) {
                if (Files.isRegularFile(entry) && isIncluded(command, entry)) {
                    var name = entry.fileName.toString()
                    if (command.name.text == "\\include" ||
                            command.name.text == "\\includesvg") {
                        name = FilenameUtils.removeExtension(name)
                    }
                    entries.add(CompletionItemFactory.createFile(name))
                } else if (Files.isDirectory(entry) && entry != directory) {
                    val name = entry.fileName.toString()
                    entries.add(CompletionItemFactory.createFolder(name))
                }
            }
            entries
        } catch (e: IOException) {
            emptyList()
        }
    }

    private fun getCurrentDirectory(request: FeatureRequest<CompletionParams>,
                                    command: LatexCommandSyntax): Path? {
        val basePath = File(request.document.uri).parentFile.toPath()
        return try {
            val include = command.extractText(0) ?: return basePath
            val relativePath = include.words.joinToString(" ") { it.text }
            val fullPath = basePath.resolve(relativePath).normalize()
            if (relativePath.endsWith("/")) {
                fullPath
            } else {
                fullPath.parent
            }
        } catch (e: InvalidPathException) {
            return null
        }
    }

    private fun isIncluded(command: LatexCommandSyntax, file: Path): Boolean {
        val allowedExtensions = when (command.name.text) {
            "\\include", "\\input" ->
                arrayOf("tex")
            "\\bibliography", "\\addbibresource" ->
                arrayOf("bib")
            "\\includegraphics" ->
                arrayOf("pdf", "png", "jpg", "jpeg", "bmp")
            "\\includesvg" ->
                arrayOf("svg")
            else ->
                arrayOf()
        }
        val extension = FilenameUtils.getExtension(file.toString())
        return allowedExtensions.contains(extension.toLowerCase())
    }
}
