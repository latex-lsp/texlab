package texlab.diagnostics

import com.google.gson.annotations.SerializedName

data class LatexLinterConfig(@SerializedName("onSave") val onSave: Boolean)
