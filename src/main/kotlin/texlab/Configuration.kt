package texlab

import com.google.gson.Gson
import com.google.gson.JsonObject
import org.eclipse.lsp4j.ConfigurationItem
import org.eclipse.lsp4j.ConfigurationParams
import org.eclipse.lsp4j.services.LanguageClient
import java.net.URI

inline fun <reified T> LanguageClient.configuration(section: String, scopeUri: URI): T {
    val item = ConfigurationItem().apply {
        this.section = section
        this.scopeUri = scopeUri.toString()
    }
    val params = ConfigurationParams(listOf(item))
    val json = configuration(params).get()[0] as JsonObject
    return Gson().fromJson(json, T::class.java) as T
}
