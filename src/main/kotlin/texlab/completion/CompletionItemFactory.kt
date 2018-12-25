package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionItemKind
import org.eclipse.lsp4j.InsertTextFormat

object CompletionItemFactory {
    private const val KERNEL = "built-in"

    fun createSnippet(name: String, unit: String?, template: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Snippet
            detail = unit ?: KERNEL
            insertText = template
            insertTextFormat = InsertTextFormat.Snippet
        }
    }

    fun createCommand(name: String, unit: String?): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Function
            detail = unit ?: KERNEL
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

    fun createFile(path: String): CompletionItem {
        return CompletionItem(path).apply {
            kind = CompletionItemKind.File
        }
    }

    fun createPgfLibrary(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Class
            commitCharacters = listOf(" ")
        }
    }

    fun createTikzLibrary(name: String): CompletionItem {
        return CompletionItem(name).apply {
            kind = CompletionItemKind.Class
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
}
