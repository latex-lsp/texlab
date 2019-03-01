package texlab

import de.undercouch.citeproc.script.ScriptRunnerFactory
import kotlinx.coroutines.ObsoleteCoroutinesApi
import org.eclipse.lsp4j.jsonrpc.Launcher

@ObsoleteCoroutinesApi
fun main() {
    ScriptRunnerFactory.setRunnerType(ScriptRunnerFactory.RunnerType.V8)
    val server = LanguageServerImpl()
    val launcher = Launcher.createLauncher(server, CustomLanguageClient::class.java, System.`in`, System.out)
    server.connect(launcher.remoteProxy)
    launcher.startListening().get()
}
