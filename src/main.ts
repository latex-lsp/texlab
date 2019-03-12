import {
  CancellationToken,
  combineFeatures,
  createConnection,
  Features,
  ProposedFeatures,
  ServerCapabilities,
  TextDocumentIdentifier,
  TextDocumentSyncKind,
} from 'vscode-languageserver';
import { BuildConfig, BuildProvider } from './build';
import { completionProvider } from './completion';
import { Document } from './document';
import { ForwardSearchConfig, forwardSearchProvider } from './forwardSearch';
import { hoverProvider } from './hover';
import { getLanguageById } from './language';
import { BuildTextDocumentRequest } from './protocol/build';
import { ForwardSearchRequest } from './protocol/forwardSearch';
import { ProgressFeature, ProgressListener } from './protocol/progress';
import { FeatureContext, FeatureProvider } from './provider';
import { Uri } from './uri';
import { Workspace } from './workspace';

const customFeatures: Features<{}, {}, {}, {}, ProgressListener> = {
  __brand: 'features',
  window: ProgressFeature,
};

const features = combineFeatures(ProposedFeatures.all, customFeatures);
const connection = createConnection(features);
const workspace = new Workspace();

const buildProvider = BuildProvider(connection.console, connection.window);

connection.onInitialize(async ({ rootUri }) => {
  if (rootUri) {
    const root = Uri.parse(rootUri);
    if (root.isFile()) {
      await workspace.loadDirectory(root);
      await workspace.loadIncludes();
      connection.console.log(workspace.documents.length.toString());
    }
  }

  const capabilities: ServerCapabilities = {
    textDocumentSync: {
      change: TextDocumentSyncKind.Full,
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
  };

  return { capabilities };
});

connection.onInitialized(() => {});
connection.onDidChangeWatchedFiles(() => {});

connection.onDidOpenTextDocument(async ({ textDocument }) => {
  const language = getLanguageById(textDocument.languageId);
  if (language === undefined) {
    return;
  }

  const uri = Uri.parse(textDocument.uri);
  const document = Document.create(uri, textDocument.text, language);
  workspace.put(document);

  await workspace.loadIncludes();
});

connection.onDidChangeTextDocument(async ({ textDocument, contentChanges }) => {
  const uri = Uri.parse(textDocument.uri);
  const text = contentChanges[0].text;
  const { tree } = workspace.documents.find(x => x.uri.equals(uri))!;
  const document = Document.create(uri, text, tree.language);
  workspace.put(document);

  await workspace.loadIncludes();
});

connection.onDidSaveTextDocument(() => {});
connection.onDocumentSymbol(() => null);
connection.onRenameRequest(() => null);
connection.onDocumentLinks(() => null);
connection.onCompletion(params => runProvider(completionProvider, params));
connection.onCompletionResolve(x => x);
connection.onFoldingRanges(() => null);
connection.onDefinition(() => null);
connection.onHover(params => runProvider(hoverProvider, params));
connection.onDocumentFormatting(() => null);
connection.onReferences(() => null);
connection.onDocumentHighlight(() => null);

connection.onRequest(
  BuildTextDocumentRequest.type,
  async (params, cancellationToken) => {
    const config: BuildConfig = await connection.workspace.getConfiguration({
      section: 'latex.build',
    });

    return runProvider(
      buildProvider,
      { ...params, ...config },
      cancellationToken,
    );
  },
);

connection.onRequest(ForwardSearchRequest.type, async params => {
  const config: ForwardSearchConfig = await connection.workspace.getConfiguration(
    {
      section: 'latex.forwardSearch',
    },
  );

  return runProvider(forwardSearchProvider, {
    ...params,
    ...config,
  });
});

connection.listen();

function runProvider<T, R>(
  provider: FeatureProvider<T, R>,
  params: { textDocument: TextDocumentIdentifier } & T,
  cancellationToken?: CancellationToken,
): Promise<R> {
  const uri = Uri.parse(params.textDocument.uri);
  const context = new FeatureContext(uri, workspace, params);
  return provider.execute(context, cancellationToken);
}
