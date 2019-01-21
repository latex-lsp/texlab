package texlab

interface ProgressListener {
    fun onReportProgress(params: ProgressParams)
}
