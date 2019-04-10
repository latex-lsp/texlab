package texlab.completion

import org.eclipse.lsp4j.*
import texlab.completion.bibtex.BibtexField
import texlab.formatting.BibtexFormatter
import texlab.syntax.bibtex.BibtexEntrySyntax

object CompletionItemFactory {
    private const val KERNEL = "built-in"

    fun createSnippet(name: String, component: String?, template: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Snippet
            detail = component ?: KERNEL
            insertText = template
            insertTextFormat = InsertTextFormat.Snippet
        }
    }

    fun createCommand(name: String, component: String?): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Function
            detail = component ?: KERNEL
        }
    }

    fun createEnvironment(name: String, unit: String?): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.EnumMember
            detail = unit ?: KERNEL
        }
    }

    fun createLabel(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Field
        }
    }

    fun createFolder(path: String): CompletionItem {
        return CompletionItem(path).apply {
            kind = CompletionItemKind.Folder
            commitCharacters = listOf("/")
        }
    }

    fun createFile(path: String): CompletionItem {
        return CompletionItem(path).apply {
            kind = CompletionItemKind.File
        }
    }

    fun createPgfLibrary(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Module
            commitCharacters = listOf(" ")
        }
    }

    fun createTikzLibrary(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Module
            commitCharacters = listOf(" ")
        }
    }

    fun createColor(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Color
        }
    }

    fun createColorModel(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Color
        }
    }

    fun createPackage(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Class
        }
    }

    fun createClass(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Class
        }
    }

    fun createCitation(entry: BibtexEntrySyntax): CompletionItem {
        return CompletionItem(entry.name!!.text).apply {
            kind = CompletionItemKind.Constant
            data = BibtexFormatter(true, 4, -1).format(entry)
        }
    }

    fun createEntryType(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Interface
        }
    }

    fun createFieldName(field: BibtexField): CompletionItem {
        return CompletionItem(field.toString()).apply {
            kind = CompletionItemKind.Field
            setDocumentation(MarkupContent().apply {
                kind = MarkupKind.MARKDOWN
                value = field.documentation()
            })
        }
    }

    fun createCommandSymbol(name: String, component: String?, image: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Function
            detail = component ?: KERNEL
            setDocumentation(createImage(name, image))
        }
    }

    fun createArgumentSymbol(name: String, image: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Field
            setDocumentation(createImage(name, image))
        }
    }

    private fun createImage(name: String, image: String): MarkupContent {
        return MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            value = "![$name](data:image/png;base64,$image|width=48,height=48)"
        }
    }
}
