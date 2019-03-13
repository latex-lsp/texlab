import {
  CompletionItem,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type CompletionProvider = FeatureProvider<
  TextDocumentPositionParams,
  CompletionItem[]
>;
