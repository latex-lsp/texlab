import * as fs from 'fs';
import * as path from 'path';
import {
  CancellationToken,
  CancellationTokenSource,
  CompletionItem,
  CompletionList,
  CompletionParams,
  Definition,
  DefinitionLink,
  Diagnostic,
  DiagnosticSeverity,
  DidChangeTextDocumentParams,
  DidChangeWatchedFilesNotification,
  DidChangeWatchedFilesParams,
  DidChangeWatchedFilesRegistrationOptions,
  DidOpenTextDocumentParams,
  DidSaveTextDocumentParams,
  DocumentFormattingParams,
  DocumentLink,
  DocumentLinkParams,
  FoldingRange,
  FoldingRangeParams,
  Hover,
  InitializeParams,
  InitializeResult,
  Location,
  Range,
  ReferenceParams,
  RenameParams,
  ServerCapabilities,
  TextDocumentIdentifier,
  TextDocumentPositionParams,
  TextDocumentSyncKind,
  TextEdit,
  WatchKind,
  WorkspaceEdit,
} from 'vscode-languageserver';
import { BuildConfig, BuildProvider } from './build';
import { CompletionProvider } from './completion';
import { CompletionItemKind } from './completion/factory';
import { LatexComponentDatabase } from './completion/latex/data/component';
import { COMPONENT_DATABASE_FILE } from './config';
import { definitonProvider } from './definition';
import { diagnosticsProvider } from './diagnostics';
import { BuildErrorKind, parseLog } from './diagnostics/buildLog';
import { LatexLinterConfig } from './diagnostics/latex';
import { Document } from './document';
import { foldingProvider } from './folding';
import { BibtexFormatter, BibtexFormatterConfig } from './formatting/bibtex';
import { ForwardSearchConfig, forwardSearchProvider } from './forwardSearch';
import { HoverProvider } from './hover';
import { getLanguageById, Language } from './language';
import { LanguageServer } from './languageServer';
import { linkProvider } from './link';
import {
  INVALID_DISTRIBUTION_MESSAGE,
  KPSEWHICH_NOT_FOUND_MESSAGE,
  UNKNOWN_DISTRIBUTION_MESSAGE,
} from './messages';
import { generateBibliography } from './metadata/bibtexEntry';
import { getComponentMetadata } from './metadata/component';
import {
  BuildResult,
  BuildTextDocumentParams,
  BuildTextDocumentRequest,
} from './protocol/build';
import {
  ForwardSearchRequest,
  ForwardSearchResult,
} from './protocol/forwardSearch';
import { FeatureContext, FeatureProvider } from './provider';
import { referenceProvider } from './references';
import { renameProvider } from './rename';
import {
  createResolver,
  TexDistributionError,
  TexDistributionErrorKind,
} from './resolver';
import { BibtexSyntaxKind } from './syntax/bibtex/ast';
import { Uri } from './uri';
import { Workspace } from './workspace';

export class LatexLanguageServer extends LanguageServer {
  private readonly workspace = new Workspace();
  private readonly resolver = createResolver();
  private readonly componentDatabase = LatexComponentDatabase.create(
    COMPONENT_DATABASE_FILE,
    this.resolver,
    this.connection.window,
  );

  private readonly completionProvider = CompletionProvider(
    this.resolver,
    this.componentDatabase,
  );

  private readonly hoverProvider = HoverProvider(this.componentDatabase);

  private readonly buildProvider = BuildProvider(
    this.connection.console,
    this.connection.window,
  );

  constructor() {
    super();

    this.resolver.catch(error => {
      if (error instanceof TexDistributionError) {
        const { window } = this.connection;
        switch (error.kind) {
          case TexDistributionErrorKind.KpsewhichNotFound:
            window.showErrorMessage(KPSEWHICH_NOT_FOUND_MESSAGE);
            break;
          case TexDistributionErrorKind.UnknownDistribution:
            window.showErrorMessage(UNKNOWN_DISTRIBUTION_MESSAGE);
            break;
          case TexDistributionErrorKind.InvalidDistribution:
            window.showErrorMessage(INVALID_DISTRIBUTION_MESSAGE);
            break;
        }
      }
    });

    this.connection.onRequest(
      BuildTextDocumentRequest.type,
      this.build.bind(this),
    );

    this.connection.onRequest(
      ForwardSearchRequest.type,
      this.forwardSearch.bind(this),
    );
  }

  public async initialize({
    rootUri,
  }: InitializeParams): Promise<InitializeResult> {
    if (rootUri) {
      const root = Uri.parse(rootUri);
      if (root.isFile()) {
        await this.workspace.loadDirectory(root);
        await this.workspace.loadIncludes();
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
  }

  public async initialized() {
    const options: DidChangeWatchedFilesRegistrationOptions = {
      watchers: [
        {
          globPattern: '**/*.log',
          kind: WatchKind.Create | WatchKind.Change,
        },
      ],
    };
    await this.connection.client.register(
      DidChangeWatchedFilesNotification.type,
      options,
    );
  }

  public async didOpenTextDocument({
    textDocument,
  }: DidOpenTextDocumentParams): Promise<void> {
    const language = getLanguageById(textDocument.languageId);
    if (language === undefined) {
      return;
    }

    const uri = Uri.parse(textDocument.uri);
    const document = Document.create(uri, textDocument.text, language);
    this.workspace.put(document);

    await this.runLinter(uri, textDocument.text);
    this.onDidOpenOrChange();
  }

  public didChangeTextDocument({
    textDocument,
    contentChanges,
  }: DidChangeTextDocumentParams): void {
    const uri = Uri.parse(textDocument.uri);
    const text = contentChanges[0].text;
    const { tree } = this.workspace.documents.find(x => x.uri.equals(uri))!;
    const document = Document.create(uri, text, tree.language);
    this.workspace.put(document);

    this.onDidOpenOrChange();
  }

  public async didSaveTextDocument({
    textDocument,
    text,
  }: DidSaveTextDocumentParams): Promise<void> {
    const uri = Uri.parse(textDocument.uri);
    await this.runLinter(uri, text!);
    await this.publishDiagnostics(uri);

    const config: BuildConfig = await this.connection.workspace.getConfiguration(
      {
        section: 'latex.build',
      },
    );
    if (config.onSave) {
      const parent = this.workspace.findParent(uri);
      if (parent !== undefined) {
        const tokenSource = new CancellationTokenSource();
        await this.build(
          { textDocument: { uri: parent.uri.toString() } },
          tokenSource.token,
        );
        tokenSource.dispose();
      }
    }
  }

  public async didChangeWatchedFiles(params: DidChangeWatchedFilesParams) {
    for (const change of params.changes) {
      const { scheme: logScheme, fsPath: logPath } = Uri.parse(change.uri);
      if (logScheme !== 'file') {
        continue;
      }

      const dirname = path.dirname(logPath);
      const texPath = path.join(
        dirname,
        path.basename(logPath).slice(0, -4) + '.tex',
      );
      const texUri = Uri.file(texPath);
      const document = this.workspace.documents.find(x => x.uri.equals(texUri));
      if (document !== undefined) {
        const diagnosticsByUri = new Map<string, Diagnostic[]>();
        diagnosticsProvider.buildProvider.diagnosticsByUri = diagnosticsByUri;
        try {
          const log = await fs.promises.readFile(logPath);
          const allErrors = parseLog(texUri, log.toString());
          allErrors
            .map(x => x.uri.toString())
            .forEach(x => diagnosticsByUri.set(x, []));

          for (const { uri, kind, message, line } of allErrors) {
            const diagnostics = diagnosticsByUri.get(uri.toString())!;
            const severity =
              kind === BuildErrorKind.Error
                ? DiagnosticSeverity.Error
                : DiagnosticSeverity.Warning;

            diagnostics.push({
              message,
              severity,
              range: Range.create(line || 0, 0, line || 0, 0),
              source: 'LaTeX',
            });
          }
        } catch {
          // File is still locked
        }
      }
    }

    for (const { uri } of this.workspace.documents) {
      await this.publishDiagnostics(uri);
    }
  }

  public async hover(
    params: TextDocumentPositionParams,
  ): Promise<Hover | undefined | null> {
    return this.runProvider(this.hoverProvider, params);
  }

  public async completion(
    params: CompletionParams,
  ): Promise<CompletionItem[] | CompletionList | undefined | null> {
    const items = await this.runProvider(this.completionProvider, params);
    const allIncludes = items.every(
      x =>
        x.kind === CompletionItemKind.Folder ||
        x.kind === CompletionItemKind.File,
    );

    return {
      isIncomplete: !allIncludes,
      items,
    };
  }

  public async completionResolve(
    item: CompletionItem,
  ): Promise<CompletionItem> {
    switch (item.data.kind) {
      case CompletionItemKind.Class:
      case CompletionItemKind.Package:
        const metadata = await getComponentMetadata(item.label);
        if (metadata !== undefined) {
          item.detail = metadata.caption;
          item.documentation = metadata.documentation;
        }
        break;
      case CompletionItemKind.Citation:
        item.documentation = generateBibliography(item.data.entry);
        break;
      default:
        break;
    }
    return item;
  }

  public async definition(
    params: TextDocumentPositionParams,
  ): Promise<Definition | DefinitionLink[] | undefined | null> {
    return this.runProvider(definitonProvider, params);
  }

  public async references(
    params: ReferenceParams,
  ): Promise<Location[] | undefined | null> {
    return this.runProvider(referenceProvider, params);
  }

  public async documentFormatting(
    params: DocumentFormattingParams,
  ): Promise<TextEdit[] | undefined | null> {
    const uri = Uri.parse(params.textDocument.uri);
    const document = this.workspace.documents.find(x => x.uri.equals(uri));
    if (document === undefined || document.tree.language !== Language.Bibtex) {
      return null;
    }

    const { insertSpaces, tabSize } = params.options;
    const config: BibtexFormatterConfig = await this.connection.workspace.getConfiguration(
      {
        section: 'bibtex.formatting',
      },
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
  }

  public async rename(
    params: RenameParams,
  ): Promise<WorkspaceEdit | undefined | null> {
    return this.runProvider(renameProvider, params);
  }

  public async documentLinks(
    params: DocumentLinkParams,
  ): Promise<DocumentLink[] | undefined | null> {
    return this.runProvider(linkProvider, params);
  }

  public async foldingRanges(
    params: FoldingRangeParams,
  ): Promise<FoldingRange[] | undefined | null> {
    return this.runProvider(foldingProvider, params);
  }

  public async build(
    params: BuildTextDocumentParams,
    cancellationToken: CancellationToken,
  ): Promise<BuildResult> {
    const config: BuildConfig = await this.connection.workspace.getConfiguration(
      {
        section: 'latex.build',
      },
    );

    return this.runProvider(
      this.buildProvider,
      { ...params, ...config },
      cancellationToken,
    );
  }

  public async forwardSearch(
    params: TextDocumentPositionParams,
  ): Promise<ForwardSearchResult> {
    const config: ForwardSearchConfig = await this.connection.workspace.getConfiguration(
      {
        section: 'latex.forwardSearch',
      },
    );

    return this.runProvider(forwardSearchProvider, {
      ...params,
      ...config,
    });
  }

  private async onDidOpenOrChange() {
    await this.workspace.loadIncludes();

    this.workspace.documents
      .map(x => this.workspace.relatedDocuments(x.uri))
      .forEach(documents =>
        this.componentDatabase.then(x => x.relatedComponents(documents)),
      );

    for (const document of this.workspace.documents) {
      await this.publishDiagnostics(document.uri);
    }
  }

  private async runLinter(uri: Uri, text: string) {
    const config: LatexLinterConfig = await this.connection.workspace.getConfiguration(
      {
        section: 'latex.lint',
      },
    );

    if (config.onSave) {
      await diagnosticsProvider.latexProvider.update(uri, text);
    } else {
      diagnosticsProvider.latexProvider.clear(uri);
    }
  }

  private async publishDiagnostics(uri: Uri) {
    const params = { textDocument: { uri: uri.toString() } };
    const diagnostics = await this.runProvider(diagnosticsProvider, params);
    await this.connection.sendDiagnostics({
      uri: uri.toString(),
      diagnostics,
    });
  }

  private async runProvider<T, R>(
    provider: FeatureProvider<T, R>,
    params: { textDocument: TextDocumentIdentifier } & T,
    cancellationToken?: CancellationToken,
  ): Promise<R> {
    const uri = Uri.parse(params.textDocument.uri);
    const context = new FeatureContext(uri, this.workspace, params);
    return provider.execute(context, cancellationToken);
  }
}
