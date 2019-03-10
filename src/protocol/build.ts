import { RequestType } from 'vscode-jsonrpc';
import { TextDocumentIdentifier } from 'vscode-languageserver';

export interface BuildTextDocumentParams {
  /**
   * The text document to build.
   */
  textDocument: TextDocumentIdentifier;
}

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
}

export interface BuildResult {
  /**
   * The status of the build process.
   */
  status: BuildStatus;
}

export abstract class BuildTextDocumentRequest {
  public static type = new RequestType<
    BuildTextDocumentParams,
    BuildResult,
    void,
    void
  >('textDocument/build');
}
