import fs from 'fs';
import os from 'os';
import path from 'path';

export const TEXLAB_HOME_DIRECTORY = path.resolve(os.homedir(), '.texlab');

export const COMPONENT_DATABASE_FILE = path.resolve(
  TEXLAB_HOME_DIRECTORY,
  'components.json',
);

if (!fs.existsSync(TEXLAB_HOME_DIRECTORY)) {
  fs.promises.mkdir(TEXLAB_HOME_DIRECTORY);
}
