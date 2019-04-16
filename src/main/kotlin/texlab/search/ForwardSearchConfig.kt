package texlab.search

import com.google.gson.annotations.SerializedName

data class ForwardSearchConfig(@SerializedName("executable") val executable: String? = null,
                               @SerializedName("args") val args: List<String> = emptyList())
