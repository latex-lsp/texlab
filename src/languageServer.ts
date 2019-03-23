import {
  CodeAction,
  CodeActionParams,
  CodeLens,
  CodeLensParams,
  ColorInformation,
  ColorPresentation,
  ColorPresentationParams,
  combineFeatures,
  Command,
  CompletionItem,
  CompletionList,
  CompletionParams,
  Connection,
  createConnection,
  Declaration,
  DeclarationLink,
  Definition,
  DefinitionLink,
  DidChangeConfigurationParams,
  DidChangeTextDocumentParams,
  DidChangeWatchedFilesParams,
  DidCloseTextDocumentParams,
  DidOpenTextDocumentParams,
  DidSaveTextDocumentParams,
  DocumentColorParams,
  DocumentFormattingParams,
  DocumentHighlight,
  DocumentLink,
  DocumentLinkParams,
  DocumentOnTypeFormattingParams,
  DocumentRangeFormattingParams,
  DocumentSymbol,
  DocumentSymbolParams,
  ExecuteCommandParams,
  Features,
  FoldingRange,
  FoldingRangeParams,
  Hover,
  InitializedParams,
  InitializeParams,
  InitializeResult,
  Location,
  ProposedFeatures,
  Range,
  ReferenceParams,
  RenameParams,
  SignatureHelp,
  SymbolInformation,
  TextDocumentPositionParams,
  TextEdit,
  WillSaveTextDocumentParams,
  WorkspaceEdit,
  WorkspaceSymbolParams,
} from 'vscode-languageserver';
import { ProgressFeature, ProgressListener } from './protocol/progress';

const CUSTOM_FEATURES: Features<{}, {}, {}, {}, ProgressListener> = {
  __brand: 'features',
  window: ProgressFeature,
};

export abstract class LanguageServer {
  protected connection: Connection<{}, {}, {}, {}, ProgressListener, {}>;

  constructor() {
    const features = combineFeatures(ProposedFeatures.all, CUSTOM_FEATURES);
    this.connection = createConnection(features);

    this.connection.onInitialize(this.initialize.bind(this));
    this.connection.onInitialized(this.initialized.bind(this));
    this.connection.onShutdown(this.shutdown.bind(this));
    this.connection.onExit(this.exit.bind(this));
    this.connection.onDidChangeConfiguration(
      this.didChangeConfiguration.bind(this),
    );
    this.connection.onDidChangeWatchedFiles(
      this.didChangeWatchedFiles.bind(this),
    );
    this.connection.onDidOpenTextDocument(this.didOpenTextDocument.bind(this));
    this.connection.onDidChangeTextDocument(
      this.didChangeTextDocument.bind(this),
    );
    this.connection.onDidCloseTextDocument(
      this.didCloseTextDocument.bind(this),
    );
    this.connection.onWillSaveTextDocument(
      this.willSaveTextDocument.bind(this),
    );
    this.connection.onWillSaveTextDocumentWaitUntil(
      this.willSaveTextDocumentWaitUntil.bind(this),
    );
    this.connection.onDidSaveTextDocument(this.didSaveTextDocument.bind(this));
    this.connection.onHover(this.hover.bind(this));
    this.connection.onCompletion(this.completion.bind(this));
    this.connection.onCompletionResolve(this.completionResolve.bind(this));
    this.connection.onSignatureHelp(this.signatureHelp.bind(this));
    this.connection.onDeclaration(this.declaration.bind(this));
    this.connection.onDefinition(this.definition.bind(this));
    this.connection.onTypeDefinition(this.typeDefinition.bind(this));
    this.connection.onImplementation(this.implementation.bind(this));
    this.connection.onReferences(this.references.bind(this));
    this.connection.onDocumentHighlight(this.documentHighlight.bind(this));
    this.connection.onDocumentSymbol(this.documentSymbol.bind(this));
    this.connection.onWorkspaceSymbol(this.workspaceSymbol.bind(this));
    this.connection.onCodeAction(this.codeAction.bind(this));
    this.connection.onCodeLens(this.codeLens.bind(this));
    this.connection.onCodeLensResolve(this.codeLensResolve.bind(this));
    this.connection.onDocumentFormatting(this.documentFormatting.bind(this));
    this.connection.onDocumentRangeFormatting(
      this.documentRangeFormatting.bind(this),
    );
    this.connection.onDocumentOnTypeFormatting(
      this.documentOnTypeFormatting.bind(this),
    );
    this.connection.onRenameRequest(this.rename.bind(this));
    this.connection.onPrepareRename(this.prepareRename.bind(this));
    this.connection.onDocumentLinks(this.documentLinks.bind(this));
    this.connection.onDocumentLinkResolve(this.documentLinkResolve.bind(this));
    this.connection.onDocumentColor(this.documentColor.bind(this));
    this.connection.onColorPresentation(this.colorPresentation.bind(this));
    this.connection.onFoldingRanges(this.foldingRanges.bind(this));
    this.connection.onExecuteCommand(this.executeCommand.bind(this));
  }

  public listen() {
    this.connection.listen();
  }

  public abstract async initialize(
    params: InitializeParams,
  ): Promise<InitializeResult>;

  public initialized(_params: InitializedParams): void {}

  public async shutdown(): Promise<void> {}

  public exit(): void {}

  public didChangeConfiguration(_params: DidChangeConfigurationParams): void {}

  public didChangeWatchedFiles(_params: DidChangeWatchedFilesParams): void {}

  public didOpenTextDocument(_params: DidOpenTextDocumentParams): void {}

  public didChangeTextDocument(_params: DidChangeTextDocumentParams): void {}

  public didCloseTextDocument(_params: DidCloseTextDocumentParams): void {}

  public willSaveTextDocument(_params: WillSaveTextDocumentParams): void {}

  public async willSaveTextDocumentWaitUntil(
    _params: WillSaveTextDocumentParams,
  ): Promise<TextEdit[] | undefined | null> {
    return undefined;
  }

  public didSaveTextDocument(_params: DidSaveTextDocumentParams): void {}

  public async hover(
    _params: TextDocumentPositionParams,
  ): Promise<Hover | undefined | null> {
    return undefined;
  }

  public async completion(
    _params: CompletionParams,
  ): Promise<CompletionItem[] | CompletionList | undefined | null> {
    return undefined;
  }

  public async completionResolve(
    item: CompletionItem,
  ): Promise<CompletionItem> {
    return item;
  }

  public async signatureHelp(
    _params: TextDocumentPositionParams,
  ): Promise<SignatureHelp | undefined | null> {
    return undefined;
  }

  public async declaration(
    _params: TextDocumentPositionParams,
  ): Promise<Declaration | DeclarationLink[] | undefined | null> {
    return undefined;
  }

  public async definition(
    _params: TextDocumentPositionParams,
  ): Promise<Definition | DefinitionLink[] | undefined | null> {
    return undefined;
  }

  public async typeDefinition(
    _params: TextDocumentPositionParams,
  ): Promise<Definition | undefined | null> {
    return undefined;
  }

  public async implementation(
    _params: TextDocumentPositionParams,
  ): Promise<Definition | undefined | null> {
    return undefined;
  }

  public async references(
    _params: ReferenceParams,
  ): Promise<Location[] | undefined | null> {
    return undefined;
  }

  public async documentHighlight(
    _params: TextDocumentPositionParams,
  ): Promise<DocumentHighlight[] | undefined | null> {
    return undefined;
  }

  public async documentSymbol(
    _params: DocumentSymbolParams,
  ): Promise<SymbolInformation[] | DocumentSymbol[] | undefined | null> {
    return undefined;
  }

  public async workspaceSymbol(
    _params: WorkspaceSymbolParams,
  ): Promise<SymbolInformation[] | undefined | null> {
    return undefined;
  }

  public async codeAction(
    _params: CodeActionParams,
  ): Promise<Array<Command | CodeAction> | undefined | null> {
    return undefined;
  }

  public async codeLens(
    _params: CodeLensParams,
  ): Promise<CodeLens[] | undefined | null> {
    return undefined;
  }

  public async codeLensResolve(item: CodeLens): Promise<CodeLens> {
    return item;
  }

  public async documentFormatting(
    _params: DocumentFormattingParams,
  ): Promise<TextEdit[] | undefined | null> {
    return undefined;
  }

  public async documentRangeFormatting(
    _params: DocumentRangeFormattingParams,
  ): Promise<TextEdit[] | undefined | null> {
    return undefined;
  }

  public async documentOnTypeFormatting(
    _params: DocumentOnTypeFormattingParams,
  ): Promise<TextEdit[] | undefined | null> {
    return undefined;
  }

  public async rename(
    _params: RenameParams,
  ): Promise<WorkspaceEdit | undefined | null> {
    return undefined;
  }

  public async prepareRename(
    _params: TextDocumentPositionParams,
  ): Promise<
    | Range
    | {
        range: Range;
        placeholder: string;
      }
    | undefined
    | null
  > {
    return undefined;
  }

  public async documentLinks(
    _params: DocumentLinkParams,
  ): Promise<DocumentLink[] | undefined | null> {
    return undefined;
  }

  public async documentLinkResolve(item: DocumentLink): Promise<DocumentLink> {
    return item;
  }

  public async documentColor(
    _params: DocumentColorParams,
  ): Promise<ColorInformation[] | undefined | null> {
    return undefined;
  }

  public async colorPresentation(
    _params: ColorPresentationParams,
  ): Promise<ColorPresentation[] | undefined | null> {
    return undefined;
  }

  public async foldingRanges(
    _params: FoldingRangeParams,
  ): Promise<FoldingRange[] | undefined | null> {
    return undefined;
  }

  public async executeCommand(
    _params: ExecuteCommandParams,
  ): Promise<any | undefined | null> {
    return undefined;
  }
}
