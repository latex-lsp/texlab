import { Diagnostic, TextDocumentIdentifier } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type DiagnosticsProvider = FeatureProvider<
  { textDocument: TextDocumentIdentifier },
  Diagnostic[]
>;
