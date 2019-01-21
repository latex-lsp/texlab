package texlab

data class ProgressParams(val id: String,
                          val title: String,
                          val message: String?,
                          val percentage: Number? = null,
                          val done: Boolean? = null)
