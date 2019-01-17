package texlab

import java.net.URI

object URIHelper {
    fun parse(text: String): URI {
        return URI.create(text.replace(" ", "%20"))
    }
}
