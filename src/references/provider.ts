import { Location, TextDocumentPositionParams } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type ReferenceProvider = FeatureProvider<
  TextDocumentPositionParams,
  Location[]
>;
