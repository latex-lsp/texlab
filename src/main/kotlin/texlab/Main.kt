package texlab

import org.eclipse.lsp4j.jsonrpc.Launcher

fun main(args: Array<String>) {
    val server = LanguageServerImpl()
    val launcher = Launcher.createLauncher(server, CustomLanguageClient::class.java, System.`in`, System.out)
    server.connect(launcher.remoteProxy)
    launcher.startListening().get()
}
