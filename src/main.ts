import {
  CancellationToken,
  combineFeatures,
  CompletionItemKind,
  createConnection,
  Features,
  ProposedFeatures,
  ServerCapabilities,
  TextDocumentIdentifier,
  TextDocumentSyncKind,
  TextEdit,
} from 'vscode-languageserver';
import { BuildConfig, BuildProvider } from './build';
import { CompletionProvider } from './completion';
import { LatexComponentDatabase } from './completion/latex/data/component';
import { COMPONENT_DATABASE_FILE } from './config';
import { definitonProvider } from './definition';
import { Document } from './document';
import { BibtexFormatter, BibtexFormatterConfig } from './formatting/bibtex';
import { ForwardSearchConfig, forwardSearchProvider } from './forwardSearch';
import { hoverProvider } from './hover/index';
import { getLanguageById, Language } from './language';
import { linkProvider } from './link';
import {
  INVALID_DISTRIBUTION_MESSAGE,
  KPSEWHICH_NOT_FOUND_MESSAGE,
  UNKNOWN_DISTRIBUTION_MESSAGE,
} from './messages';
import { BuildTextDocumentRequest } from './protocol/build';
import { ForwardSearchRequest } from './protocol/forwardSearch';
import { ProgressFeature, ProgressListener } from './protocol/progress';
import { FeatureContext, FeatureProvider } from './provider';
import {
  createResolver,
  TexDistributionError,
  TexDistributionErrorKind,
} from './resolver';
import { BibtexSyntaxKind } from './syntax/bibtex/ast';
import { Uri } from './uri';
import { Workspace } from './workspace';

const customFeatures: Features<{}, {}, {}, {}, ProgressListener> = {
  __brand: 'features',
  window: ProgressFeature,
};

const features = combineFeatures(ProposedFeatures.all, customFeatures);
const connection = createConnection(features);
const workspace = new Workspace();

const resolver = createResolver();
resolver.catch(error => {
  if (error instanceof TexDistributionError) {
    switch (error.kind) {
      case TexDistributionErrorKind.KpsewhichNotFound:
        connection.window.showErrorMessage(KPSEWHICH_NOT_FOUND_MESSAGE);
        break;
      case TexDistributionErrorKind.UnknownDistribution:
        connection.window.showErrorMessage(UNKNOWN_DISTRIBUTION_MESSAGE);
        break;
      case TexDistributionErrorKind.InvalidDistribution:
        connection.window.showErrorMessage(INVALID_DISTRIBUTION_MESSAGE);
        break;
    }
  }
});

const componentDatabase = LatexComponentDatabase.create(
  COMPONENT_DATABASE_FILE,
  resolver,
  connection.window,
);

const completionProvider = CompletionProvider(resolver, componentDatabase);
const buildProvider = BuildProvider(connection.console, connection.window);

connection.onInitialize(async ({ rootUri }) => {
  if (rootUri) {
    const root = Uri.parse(rootUri);
    if (root.isFile()) {
      await workspace.loadDirectory(root);
      await workspace.loadIncludes();
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

  await onDidOpenOrChange();
});

connection.onDidChangeTextDocument(async ({ textDocument, contentChanges }) => {
  const uri = Uri.parse(textDocument.uri);
  const text = contentChanges[0].text;
  const { tree } = workspace.documents.find(x => x.uri.equals(uri))!;
  const document = Document.create(uri, text, tree.language);
  workspace.put(document);

  await onDidOpenOrChange();
});

connection.onDidSaveTextDocument(() => {});
connection.onDocumentSymbol(() => null);
connection.onRenameRequest(() => null);
connection.onDocumentLinks(params => runProvider(linkProvider, params));

connection.onCompletion(async params => {
  const items = await runProvider(completionProvider, params);
  const allIncludes = items.every(
    x =>
      x.kind === CompletionItemKind.Folder ||
      x.kind === CompletionItemKind.File,
  );
  return {
    isIncomplete: !allIncludes,
    items,
  };
});

connection.onCompletionResolve(x => x);
connection.onFoldingRanges(() => null);
connection.onDefinition(params => runProvider(definitonProvider, params));
connection.onHover(params => runProvider(hoverProvider, params));

connection.onDocumentFormatting(async params => {
  const uri = Uri.parse(params.textDocument.uri);
  const document = workspace.documents.find(x => x.uri.equals(uri));
  if (document === undefined || document.tree.language !== Language.Bibtex) {
    return null;
  }

  const { insertSpaces, tabSize } = params.options;
  const config: BibtexFormatterConfig = await connection.workspace.getConfiguration(
    { section: 'bibtex.formatting' },
  );

  const formatter = new BibtexFormatter(
    insertSpaces,
    tabSize,
    config.lineLength,
  );

  const edits: TextEdit[] = [];
  document.tree.root.children.forEach(declaration => {
    switch (declaration.kind) {
      case BibtexSyntaxKind.Preamble:
      case BibtexSyntaxKind.String:
      case BibtexSyntaxKind.Entry:
        edits.push({
          range: declaration.range,
          newText: formatter.format(declaration),
        });
        break;
    }
  });
  return edits;
});

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

async function onDidOpenOrChange() {
  await workspace.loadIncludes();

  workspace.documents
    .map(x => workspace.relatedDocuments(x.uri))
    .forEach(documents =>
      componentDatabase.then(x => x.relatedComponents(documents)),
    );
}
