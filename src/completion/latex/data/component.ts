import PromiseQueue from 'easy-promise-queue';
import * as fs from 'fs';
import * as path from 'path';
import { Document } from '../../../document';
import { Language } from '../../../language';
import { ProgressListener, ProgressParams } from '../../../protocol/progress';
import { TexResolver } from '../../../resolver';
import { LatexComponentAnalyzer } from './analyzer';

export interface LatexComponent {
  fileNames: string[];
  references: string[];
  commands: string[];
  environments: string[];
}

export class LatexComponentDatabase {
  public static async create(
    databaseFile: string,
    resolver: Promise<TexResolver>,
    listener: ProgressListener,
  ): Promise<LatexComponentDatabase> {
    let components: LatexComponent[] = [];
    if (fs.existsSync(databaseFile)) {
      const buffer = await fs.promises.readFile(databaseFile);
      components = JSON.parse(buffer.toString()) as LatexComponent[];
    }

    return new LatexComponentDatabase(
      databaseFile,
      await resolver,
      listener,
      components,
    );
  }

  private componentsByName = new Map<string, LatexComponent>();
  private analyzer: LatexComponentAnalyzer;
  private queue = new PromiseQueue({ concurrency: 1 });

  constructor(
    private databaseFile: string,
    private resolver: TexResolver,
    private listener: ProgressListener,
    components: LatexComponent[],
  ) {
    for (const component of components) {
      component.fileNames.forEach(x => this.componentsByName.set(x, component));
    }

    this.analyzer = new LatexComponentAnalyzer(resolver, this.componentsByName);
  }

  public getComponent(fileName: string): LatexComponent | undefined {
    const component = this.componentsByName.get(fileName);
    if (component !== undefined) {
      return component;
    }

    const file = this.resolver.filesByName.get(fileName);
    if (file !== undefined) {
      this.queue.add(() => this.analyze(file));
    }

    return undefined;
  }

  public relatedComponents(documents: Document[]): LatexComponent[] {
    const components = new Set<LatexComponent>();
    for (const { tree } of documents) {
      if (tree.language === Language.Latex) {
        tree.components
          .map(x => this.getComponent(x))
          .filter((x): x is LatexComponent => x !== undefined)
          .forEach(x => components.add(x));
      }
    }

    for (const component of components) {
      const related = [
        component,
        ...component.references
          .map(x => this.getComponent(x))
          .filter((x): x is LatexComponent => x !== undefined),
      ];

      related.forEach(x => components.add(x));
    }

    return [...components];
  }

  private async analyze(file: string) {
    const fileName = path.basename(file);
    if (this.componentsByName.has(fileName)) {
      return;
    }

    const progress: ProgressParams = {
      id: 'index',
      title: 'Indexing...',
      message: fileName,
    };
    this.listener.progress(progress);

    const components = await this.analyzer.findComponents(file);
    for (const units of components) {
      this.listener.progress({
        ...progress,
        message: path.basename(units[0].file),
      });

      const component = await this.analyzer.analyzeComponent(units);
      this.addAndSave(component);
    }

    this.listener.progress({ ...progress, done: true });
  }

  private async addAndSave(component: LatexComponent) {
    for (const fileName of component.fileNames) {
      this.componentsByName.set(fileName, component);
    }

    const json = JSON.stringify([...this.componentsByName.values()], null, 2);
    await fs.promises.writeFile(this.databaseFile, json);
  }
}
