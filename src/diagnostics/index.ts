import {
  CancellationToken,
  Diagnostic,
  TextDocumentIdentifier,
} from 'vscode-languageserver';
import { concat, FeatureContext } from '../provider';
import { BibtexEntryDiagnosticsProvider } from './bibtexEntry';
import { LatexDiagnosticsProvider } from './latex';
import { DiagnosticsProvider } from './provider';

class DefaultDiagnosticsProvider implements DiagnosticsProvider {
  public readonly latexProvider: LatexDiagnosticsProvider;
  private readonly allProviders: DiagnosticsProvider;

  constructor() {
    this.latexProvider = new LatexDiagnosticsProvider();
    this.allProviders = concat(
      BibtexEntryDiagnosticsProvider,
      this.latexProvider,
    );
  }

  public execute(
    context: FeatureContext<{ textDocument: TextDocumentIdentifier }>,
    cancellationToken?: CancellationToken,
  ): Promise<Diagnostic[]> {
    return this.allProviders.execute(context, cancellationToken);
  }
}

export const diagnosticsProvider = new DefaultDiagnosticsProvider();
