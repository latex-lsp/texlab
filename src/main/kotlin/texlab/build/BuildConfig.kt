package texlab.build

import com.google.gson.annotations.SerializedName

data class BuildConfig(@SerializedName("executable") val executable: String = "latexmk",
                       @SerializedName("args") val args: List<String> = listOf("-pdf", "-interaction=nonstopmode", "-synctex=1"),
                       @SerializedName("onSave") val onSave: Boolean = false)
