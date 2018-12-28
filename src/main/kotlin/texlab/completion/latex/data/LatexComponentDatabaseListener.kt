package texlab.completion.latex.data

import java.io.File

interface LatexComponentDatabaseListener {
    fun onStartIndexing(file: File)

    fun onStopIndexing()
}
