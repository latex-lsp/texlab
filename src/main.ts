import {
  combineFeatures,
  createConnection,
  Features,
  ProposedFeatures,
  TextDocuments,
} from 'vscode-languageserver';
import { ProgressFeature, ProgressListener } from './protocol/progress';
import { Workspace } from './workspace';

const customFeatures: Features<{}, {}, {}, {}, ProgressListener> = {
  __brand: 'features',
  window: ProgressFeature,
};

const features = combineFeatures(ProposedFeatures.all, customFeatures);
const connection = createConnection(features);
const documents = new TextDocuments();
const workspace = new Workspace();

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

documents.listen(connection);
connection.listen();
