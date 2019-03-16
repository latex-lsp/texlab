import * as program from 'commander';
import * as metadata from '../package.json';
import { LatexLanguageServer } from './latexLanguageServer';

program
  .description(metadata.description)
  .version(metadata.version)
  .option('--node-ipc')
  .option('--stdio')
  .option('--socket')
  .on('command:*', () => {
    program.outputHelp();
    process.exit(1);
  });

program.parse(process.argv);

const server = new LatexLanguageServer();
server.listen();
