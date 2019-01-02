package texlab.formatting

import com.google.gson.annotations.SerializedName

enum class NamingStyle {
    @SerializedName("upper-case")
    UPPER_CASE,

    @SerializedName("lower-case")
    LOWER_CASE,

    @SerializedName("title-case")
    TITLE_CASE;

    fun format(identifier: String): String {
        if (name == "") {
            return ""
        }

        return when (this) {
            UPPER_CASE -> identifier.toUpperCase()
            LOWER_CASE -> identifier.toLowerCase()
            TITLE_CASE -> identifier[0].toUpperCase() + identifier.toLowerCase().substring(1)
        }
    }

    fun formatType(type: String): String {
        val name = type.substring(1)
        return "@" + format(name)
    }
}
