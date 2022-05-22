import * as vscode from 'vscode';
import {
  BaseLanguageClient,
  ClientCapabilities,
  DynamicFeature,
  FeatureState,
  LanguageClient,
  LanguageClientOptions,
  RequestType,
  ServerOptions,
  StaticFeature,
  TextDocumentIdentifier,
  TextDocumentPositionParams,
  WorkDoneProgress,
  WorkDoneProgressCreateRequest,
} from 'vscode-languageclient/node';
import { ExtensionState, StatusIcon } from './view';

export enum BuildStatus {
  /**
   * The build process terminated without any errors.
   */
  Success = 0,

  /**
   * The build process terminated with errors.
   */
  Error = 1,

  /**
   * The build process failed to start or crashed.
   */
  Failure = 2,

  /**
   * The build process was cancelled.
   */
  Cancelled = 3,
}

export interface BuildResult {
  /**
   * The status of the build process.
   */
  status: BuildStatus;
}

export enum ForwardSearchStatus {
  /**
   * The previewer process executed the command without any errors.
   */
  Success = 0,

  /**
   * The previewer process executed the command with errors.
   */
  Error = 1,

  /**
   * The previewer process failed to start or crashed.
   */
  Failure = 2,

  /**
   * The previewer command is not configured.
   */
  Unconfigured = 3,
}

export interface ForwardSearchResult {
  /**
   * The status of the previewer process.
   */
  status: ForwardSearchStatus;
}

interface BuildTextDocumentParams {
  /**
   * The text document to build.
   */
  textDocument: TextDocumentIdentifier;
}

abstract class BuildTextDocumentRequest {
  public static type = new RequestType<
    BuildTextDocumentParams,
    BuildResult,
    void
  >('textDocument/build');
}

abstract class ForwardSearchRequest {
  public static type = new RequestType<
    TextDocumentPositionParams,
    ForwardSearchResult,
    void
  >('textDocument/forwardSearch');
}

export class CustomProgressFeature implements StaticFeature {
  public fillClientCapabilities(capabilities: ClientCapabilities): void {
    if (!capabilities.window) {
      capabilities.window = {};
    }
    capabilities.window.workDoneProgress = true;
  }

  constructor(
    private readonly client: BaseLanguageClient,
    private readonly icon: StatusIcon,
  ) {}

  public getState(): FeatureState {
    return { kind: 'static' };
  }

  public dispose(): void {
    // nothing to dispose here
  }

  public initialize(): void {
    this.client.onRequest(WorkDoneProgressCreateRequest.type, ({ token }) => {
      this.icon.update(ExtensionState.Building);
      this.client.onProgress(WorkDoneProgress.type, token, (progress) => {
        if (progress.kind === 'end') {
          this.icon.update(ExtensionState.Running);
        }
      });
    });
  }
}

export class LatexLanguageClient extends LanguageClient {
  constructor(
    name: string,
    serverOptions: ServerOptions,
    clientOptions: LanguageClientOptions,
    icon: StatusIcon,
  ) {
    super(name, serverOptions, clientOptions);
    this.registerProposedFeatures();
    this.registerFeature(new CustomProgressFeature(this, icon));
  }

  public registerFeature(
    feature: StaticFeature | DynamicFeature<unknown>,
  ): void {
    if (feature.constructor.name !== 'ProgressFeature') {
      super.registerFeature(feature);
    }
  }

  public async build(document: vscode.TextDocument): Promise<BuildResult> {
    return await this.sendRequest(BuildTextDocumentRequest.type, {
      textDocument:
        this.code2ProtocolConverter.asTextDocumentIdentifier(document),
    });
  }

  public async forwardSearch(
    document: vscode.TextDocument,
    position: vscode.Position,
  ): Promise<ForwardSearchResult> {
    const params = this.code2ProtocolConverter.asTextDocumentPositionParams(
      document,
      position,
    );

    return this.sendRequest(ForwardSearchRequest.type, params);
  }
}
