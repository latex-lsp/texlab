import {
  CancellationToken,
  Diagnostic,
  DiagnosticSeverity,
  Range,
  TextDocumentIdentifier,
} from 'vscode-languageserver';
import { Language } from '../language';
import { ProcessBuilder, ProcessStatus } from '../process';
import { FeatureContext } from '../provider';
import { Uri } from '../uri';
import { DiagnosticsProvider } from './provider';

export interface LatexLinterConfig {
  onSave: boolean;
}

export class LatexDiagnosticsProvider implements DiagnosticsProvider {
  private readonly diagnosticsByUri: Map<string, Diagnostic[]>;

  constructor() {
    this.diagnosticsByUri = new Map();
  }

  public async execute(
    context: FeatureContext<{ textDocument: TextDocumentIdentifier }>,
  ): Promise<Diagnostic[]> {
    const { uri, document } = context;
    if (document.tree.language !== Language.Latex) {
      return [];
    }

    return this.diagnosticsByUri.get(uri.toString()) || [];
  }

  public async update(
    uri: Uri,
    text: string,
    cancellationToken?: CancellationToken,
  ) {
    const diagnostics = await lint(text, cancellationToken);
    this.diagnosticsByUri.set(uri.toString(), diagnostics);
  }

  public clear(uri: Uri) {
    this.diagnosticsByUri.delete(uri.toString());
  }
}

const LINE_REGEX = /(\d+):(\d+):(\d+):(\w+):(\w)+:(.*)/g;

export async function lint(
  text: string,
  cancellationToken?: CancellationToken,
): Promise<Diagnostic[]> {
  let stdout = '';
  const process = new ProcessBuilder('chktex')
    .args('-I0', '-f%l:%c:%d:%k:%n:%m\n')
    .output(data => (stdout += data.toString()))
    .error(() => {})
    .input(text);

  const status = await process.start(cancellationToken);
  const diagnostics: Diagnostic[] = [];
  if (status === ProcessStatus.Success) {
    stdout
      .split(/\r?\n/)
      .filter(x => x !== '')
      .map(x => LINE_REGEX.exec(x))
      .forEach(match => {
        if (match) {
          const line = parseInt(match[1], 10) - 1;
          const character = parseInt(match[2], 10) - 1;
          const digit = parseInt(match[3], 10);
          const kind = match[4];
          const code = match[5];
          const message = match[6];
          const range = Range.create(line, character, line, character + digit);
          let severity: DiagnosticSeverity;
          switch (kind) {
            case 'Message':
              severity = DiagnosticSeverity.Information;
              break;
            case 'Warning':
              severity = DiagnosticSeverity.Warning;
              break;
            default:
              severity = DiagnosticSeverity.Error;
              break;
          }
          diagnostics.push({
            range,
            code,
            message,
            severity,
            source: 'ChkTeX',
          });
        }
      });
    return diagnostics;
  }

  return [];
}
