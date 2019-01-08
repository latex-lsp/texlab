package texlab.forwardSearch

import com.google.gson.annotations.SerializedName

data class ForwardSearchConfig(@SerializedName("executable") val executable: String?,
                               @SerializedName("args") val args: List<String>)
