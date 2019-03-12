import * as cp from 'child_process';
import * as path from 'path';
import { TextDocumentPositionParams } from 'vscode-languageserver';
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

const SUCCESS_DELAY = 250;

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

    const process = cp.spawn(executable, args.map(replacePlaceholder));
    return new Promise(resolve => {
      process.on('error', () =>
        resolve({ status: ForwardSearchStatus.Failure }),
      );

      setTimeout(
        () => resolve({ status: ForwardSearchStatus.Success }),
        SUCCESS_DELAY,
      );
    });
  },
};
