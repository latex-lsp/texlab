import {
  CancellationToken,
  combineFeatures,
  createConnection,
  Features,
  ProposedFeatures,
  TextDocumentIdentifier,
  TextDocuments,
} from 'vscode-languageserver';
import { BuildConfig, BuildFeature } from './build';
import { FeatureContext, LanguageFeature } from './feature';
import { BuildTextDocumentRequest } from './protocol/build';
import { ProgressFeature, ProgressListener } from './protocol/progress';
import { Uri } from './uri';
import { Workspace } from './workspace';

const customFeatures: Features<{}, {}, {}, {}, ProgressListener> = {
  __brand: 'features',
  window: ProgressFeature,
};

const features = combineFeatures(ProposedFeatures.all, customFeatures);
const connection = createConnection(features);
const documents = new TextDocuments();
const workspace = new Workspace();

const buildFeature = new BuildFeature(connection.console, connection.window);

connection.onInitialize(() => {
  return {
    capabilities: {
      textDocumentSync: {
        change: documents.syncKind,
        save: { includeText: true },
        openClose: true,
      },
      documentSymbolProvider: true,
      renameProvider: true,
      documentLinkProvider: { resolveProvider: false },
      completionProvider: {
        resolveProvider: true,
        triggerCharacters: ['\\', '{', '}', '@', '/'],
      },
      foldingRangeProvider: true,
      definitionProvider: true,
      hoverProvider: true,
      documentFormattingProvider: true,
      referencesProvider: true,
      documentHighlightProvider: true,
    },
  };
});

connection.onInitialized(() => {});
connection.onDidChangeWatchedFiles(() => {});
connection.onDidOpenTextDocument(() => {});
connection.onDidChangeTextDocument(() => {});
connection.onDidSaveTextDocument(() => {});
connection.onDidCloseTextDocument(() => {});
connection.onDocumentSymbol(() => null);
connection.onRenameRequest(() => null);
connection.onDocumentLinks(() => null);
connection.onCompletion(() => null);
connection.onCompletionResolve(x => x);
connection.onFoldingRanges(() => null);
connection.onDefinition(() => null);
connection.onHover(() => null);
connection.onDocumentFormatting(() => null);
connection.onReferences(() => null);
connection.onDocumentHighlight(() => null);
connection.onRequest(
  BuildTextDocumentRequest.type,
  async ({ textDocument }, cancellationToken) => {
    const config: BuildConfig = await connection.workspace.getConfiguration({
      section: 'latex.build',
    });

    return runFeature(buildFeature, textDocument, config, cancellationToken);
  },
);

documents.listen(connection);
connection.listen();

function runFeature<T, R>(
  feature: LanguageFeature<T, R>,
  document: TextDocumentIdentifier,
  params: T,
  cancellationToken?: CancellationToken,
): Promise<R> {
  const uri = Uri.parse(document.uri);
  const context = new FeatureContext(uri, workspace, params);
  return feature.execute(context, cancellationToken);
}
