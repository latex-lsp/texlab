package texlab.build

import java.net.URI

data class BuildError(val uri: URI,
                      val kind: BuildErrorKind,
                      val message: String,
                      val line: Int?)

