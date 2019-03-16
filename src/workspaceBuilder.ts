import * as path from 'path';
import { TextDocumentPositionParams } from 'vscode-languageserver';
import { Document } from './document';
import { getLanguageByExtension } from './language';
import { FeatureContext, FeatureProvider } from './provider';
import { Uri } from './uri';
import { Workspace } from './workspace';

export class WorkspaceBuilder {
  public readonly workspace: Workspace;

  constructor() {
    this.workspace = new Workspace();
  }

  public document(file: string, text: string): Uri {
    const uri = Uri.file(path.resolve(file));
    const language = getLanguageByExtension(path.extname(file));
    const document = Document.create(uri, text, language!);
    this.workspace.put(document);
    return uri;
  }

  public context(
    uri: Uri,
    line: number,
    character: number,
  ): FeatureContext<TextDocumentPositionParams> {
    const params: TextDocumentPositionParams = {
      position: { line, character },
      textDocument: {
        uri: uri.toString(true),
      },
    };
    return new FeatureContext(uri, this.workspace, params);
  }
}

export interface SingleFileRunOptions<T, R> {
  provider: FeatureProvider<T, R>;
  file: string;
  text: string;
  line: number;
  character: number;
}

export function runSingleFile<R>({
  provider,
  file,
  text,
  line,
  character,
}: SingleFileRunOptions<TextDocumentPositionParams, R>): Promise<R> {
  const builder = new WorkspaceBuilder();
  const uri = builder.document(file, text);
  const context = builder.context(uri, line, character);
  return provider.execute(context);
}
