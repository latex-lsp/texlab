package texlab

import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

object LatexLanguageServerConfig {
    private val HOME_DIRECTORY: Path = Paths.get(System.getProperty("user.home"))

    private val COMPONENT_DATABASE_DIRECTORY: Path = HOME_DIRECTORY.resolve(".texlab")

    val COMPONENT_DATABASE_FILE: Path = COMPONENT_DATABASE_DIRECTORY.resolve("components.json")

    const val COMPLETION_LIMIT: Int = 50

    init {
        if (!Files.exists(COMPONENT_DATABASE_DIRECTORY)) {
            Files.createDirectory(COMPONENT_DATABASE_DIRECTORY)
        }
    }
}
