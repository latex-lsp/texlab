import {
  CancellationToken,
  Diagnostic,
  TextDocumentIdentifier,
} from 'vscode-languageserver';
import { concat, FeatureContext } from '../provider';
import { BibtexEntryDiagnosticsProvider } from './bibtexEntry';
import { DiagnosticsProvider } from './provider';

class DefaultDiagnosticsProvider implements DiagnosticsProvider {
  private readonly provider = concat(BibtexEntryDiagnosticsProvider);

  public execute(
    context: FeatureContext<{ textDocument: TextDocumentIdentifier }>,
    cancellationToken?: CancellationToken,
  ): Promise<Diagnostic[]> {
    return this.provider.execute(context, cancellationToken);
  }
}

export const diagnosticsProvider = new DefaultDiagnosticsProvider();
