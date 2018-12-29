package texlab.build

import com.google.gson.annotations.SerializedName

data class BuildConfig(@SerializedName("executable") val executable: String,
                       @SerializedName("args") val args: List<String>)
