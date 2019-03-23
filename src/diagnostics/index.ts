import {
  CancellationToken,
  Diagnostic,
  TextDocumentIdentifier,
} from 'vscode-languageserver';
import { concat, FeatureContext } from '../provider';
import { BibtexEntryDiagnosticsProvider } from './bibtexEntry';
import { LatexDiagnosticsProvider } from './latex';
import { ManualDiagnosticsProvider } from './manual';
import { DiagnosticsProvider } from './provider';

class DefaultDiagnosticsProvider implements DiagnosticsProvider {
  public readonly latexProvider: LatexDiagnosticsProvider;
  public readonly buildProvider: ManualDiagnosticsProvider;
  private readonly allProviders: DiagnosticsProvider;

  constructor() {
    this.latexProvider = new LatexDiagnosticsProvider();
    this.buildProvider = new ManualDiagnosticsProvider();
    this.allProviders = concat(
      BibtexEntryDiagnosticsProvider,
      this.latexProvider,
      this.buildProvider,
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
