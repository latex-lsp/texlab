package texlab

import java.net.URI

object URIHelper {
    fun parse(text: String): URI {
        var newText = text
        newText = normalizeDriveLetter(newText)
        newText = newText.replace(" ", "%20")
        return URI.create(newText)
    }

    fun normalizeDriveLetter(text: String): String {
        var newText = text
        for (c in 'A'..'Z') {
            newText = newText.replace("$c:/", "${c.toLowerCase()}:/")
        }
        return newText
    }
}
