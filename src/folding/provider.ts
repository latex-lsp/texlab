import { FoldingRange, FoldingRangeParams } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type FoldingProvider = FeatureProvider<
  FoldingRangeParams,
  FoldingRange[]
>;
