import path from 'path';
import {
  CancellationTokenSource,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { ProcessBuilder, ProcessStatus } from './process';
import {
  ForwardSearchResult,
  ForwardSearchStatus,
} from './protocol/forwardSearch';
import { FeatureProvider } from './provider';

export interface ForwardSearchConfig {
  executable?: string;
  args: string[];
}

export type ForwardSearchProvider = FeatureProvider<
  TextDocumentPositionParams & ForwardSearchConfig,
  ForwardSearchResult
>;

function toForwardSearchResult(status: ProcessStatus): ForwardSearchResult {
  switch (status) {
    case ProcessStatus.Success:
    case ProcessStatus.Error:
      return { status: ForwardSearchStatus.Success };
    case ProcessStatus.Failure:
      return { status: ForwardSearchStatus.Failure };
  }
}

const WAIT_TIME = 1000;

export const forwardSearchProvider: ForwardSearchProvider = {
  execute: async context => {
    const { uri, params, workspace } = context;
    const { position, executable, args } = params;

    if (!executable) {
      return { status: ForwardSearchStatus.Unconfigured };
    }

    const parentFile = workspace.findParent(uri)!.uri.fsPath;
    const pdfFile = path.join(
      path.dirname(parentFile),
      path.basename(parentFile, path.extname(parentFile)) + '.pdf',
    );

    const replacePlaceholder = (argument: string) => {
      if (argument.startsWith('"') && argument.endsWith('"')) {
        return argument;
      }

      return argument
        .replace('%f', parentFile)
        .replace('%p', pdfFile)
        .replace('%l', position.line.toString());
    };

    const cancellationTokenSource = new CancellationTokenSource();
    setTimeout(() => {
      cancellationTokenSource.cancel();
    }, WAIT_TIME);

    try {
      return toForwardSearchResult(
        await new ProcessBuilder(executable)
          .args(...args.map(replacePlaceholder))
          .start(cancellationTokenSource.token, true),
      );
    } catch {
      return { status: ForwardSearchStatus.Success };
    } finally {
      cancellationTokenSource.dispose();
    }
  },
};
