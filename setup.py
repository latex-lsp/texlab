import urllib.request
import shutil
import subprocess

if __name__ == '__main__':
    print('Downloading completion database...')
    database_url = 'https://github.com/latex-lsp/latex-completion-data/releases/download/v19.07.1/completion.json'
    with urllib.request.urlopen(database_url) as response, open('src/completion/data.json', 'wb') as out_file:
        shutil.copyfileobj(response, out_file)

    print('Bundling citeproc...')
    citeproc_dir = 'src/citeproc/script'
    subprocess.run('npm ci && npm run dist', shell=True, cwd=citeproc_dir)
