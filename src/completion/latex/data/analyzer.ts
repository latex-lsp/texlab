import path from 'path';
import { TexResolver } from '../../../resolver';
import { KERNEL_COMMANDS, KERNEL_ENVIRONMENTS } from '../../kernel';
import { LatexComponent } from './component';
import { stronglyConnectedComponents } from './graph';
import { LatexUnit } from './latexUnit';

export class LatexComponentAnalyzer {
  constructor(
    private resolver: TexResolver,
    private componentsByName: ReadonlyMap<string, LatexComponent>,
  ) {}

  public async analyzeComponent(units: LatexUnit[]): Promise<LatexComponent> {
    const unit = units[0];
    const candidates = new Set(unit.likelyPrimitives);
    KERNEL_COMMANDS.forEach(x => candidates.delete(x));
    KERNEL_ENVIRONMENTS.forEach(x => candidates.delete(x));

    const references = unit.references.map(x => path.basename(x));
    references
      .map(x => this.componentsByName.get(x))
      .filter((x): x is LatexComponent => x !== undefined)
      .forEach(reference => {
        reference.commands.forEach(x => candidates.delete(x));
        reference.environments.forEach(x => candidates.delete(x));
      });

    const fileNames = units.map(x => path.basename(x.file));
    const { commands, environments } = await unit.checkPrimitives([
      ...candidates,
    ]);

    return {
      fileNames,
      references,
      commands,
      environments,
    };
  }

  public async findComponents(file: string): Promise<LatexUnit[][]> {
    const unitsByFile = new Map<string, LatexUnit>();
    const unit = await LatexUnit.load(file, this.resolver);
    unitsByFile.set(unit.file, unit);

    const references = await Promise.all(
      unit.references
        .filter(x => !this.componentsByName.has(path.basename(x)))
        .map(x => LatexUnit.load(x, this.resolver)),
    );

    references.forEach(x => unitsByFile.set(x.file, x));

    return stronglyConnectedComponents([...unitsByFile.values()], vertex =>
      vertex.references
        .map(x => unitsByFile.get(x))
        .filter((x): x is LatexUnit => x !== undefined),
    );
  }
}
