import { RequestType, TextDocumentPositionParams } from 'vscode-languageserver';

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

export abstract class ForwardSearchRequest {
  public static type = new RequestType<
    TextDocumentPositionParams,
    ForwardSearchResult,
    void,
    void
  >('textDocument/forwardSearch');
}
