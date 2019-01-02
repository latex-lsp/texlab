package texlab.formatting

import com.google.gson.annotations.SerializedName

data class BibtexStyle(@SerializedName("types") val types: NamingStyle,
                       @SerializedName("fields") val fields: NamingStyle,
                       @SerializedName("lineLength") val lineLength: Int)
