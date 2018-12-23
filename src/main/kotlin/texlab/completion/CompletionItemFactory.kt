package texlab.completion

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionItemKind

class CompletionItemFactory {
    companion object {
        private const val KERNEL = "built-in"

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
    }
}
