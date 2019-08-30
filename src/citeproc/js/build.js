const cp = require('child_process');
const fs = require('fs');
const path = require('path');

function shouldBuild() {
  if (!fs.existsSync(path.join(__dirname, 'dist', 'citeproc.js'))) {
    return true;
  }

  let mtime = 0;
  fs.readdirSync(path.join(__dirname))
    .map(fs.lstatSync)
    .filter(lstat => lstat.isFile())
    .forEach(lstat => {
      mtime = Math.max(mtime, lstat.mtimeMs);
    });

  const lstat = fs.lstatSync(path.join(__dirname, 'dist', 'citeproc.js'));
  return mtime > lstat.mtimeMs;
}

if (shouldBuild()) {
  cp.execSync('npm install && npm run dist', {
    cwd: __dirname,
    stdio: 'inherit',
  });
}
