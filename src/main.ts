import 'array-flat-polyfill';
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
import { BuildConfig, BuildFeature } from './build';
import { Document } from './document';
import { FeatureContext, LanguageFeature } from './feature';
import { ForwardSearchConfig, ForwardSearchFeature } from './forwardSearch';
import { getLanguageById } from './language';
import { BuildTextDocumentRequest } from './protocol/build';
import { ForwardSearchRequest } from './protocol/forwardSearch';
import { ProgressFeature, ProgressListener } from './protocol/progress';
import { Uri } from './uri';
import { Workspace } from './workspace';

const customFeatures: Features<{}, {}, {}, {}, ProgressListener> = {
  __brand: 'features',
  window: ProgressFeature,
};

const features = combineFeatures(ProposedFeatures.all, customFeatures);
const connection = createConnection(features);
const workspace = new Workspace();

const buildFeature = new BuildFeature(connection.console, connection.window);
const forwardSearchFeature = new ForwardSearchFeature();

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

connection.onRequest(ForwardSearchRequest.type, async params => {
  const config: ForwardSearchConfig = await connection.workspace.getConfiguration(
    {
      section: 'latex.forwardSearch',
    },
  );

  return runFeature(forwardSearchFeature, params.textDocument, {
    ...params,
    ...config,
  });
});

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
