import { Diagnostic, TextDocumentIdentifier } from 'vscode-languageserver';
import { FeatureContext } from '../provider';
import { DiagnosticsProvider } from './provider';

export class ManualDiagnosticsProvider implements DiagnosticsProvider {
  public diagnosticsByUri: Map<string, Diagnostic[]> = new Map();

  public async execute(
    context: FeatureContext<{ textDocument: TextDocumentIdentifier }>,
  ): Promise<Diagnostic[]> {
    return this.diagnosticsByUri.get(context.uri.toString()) || [];
  }
}
