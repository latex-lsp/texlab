package texlab.metadata

import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import java.io.IOException
import java.net.URL

class CtanPackageMetadataProvider : PackageMetadataProvider {
    override fun getMetadata(name: String): PackageMetadata? {
        return try {
            val json = URL("https://ctan.org/json/2.0/pkg/$name").readText()
            val mapper = jacksonObjectMapper()
                    .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
            return mapper.readValue<PackageMetadata>(json)
        } catch (e: IOException) {
            null
        }
    }
}
