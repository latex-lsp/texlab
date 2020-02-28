use std::{error, fmt, io, process::Stdio, time::Duration};
use tempfile::{tempdir, TempDir};
use tokio::{
    fs,
    process::Command,
    time::{timeout, Elapsed},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Latex,
    Pdflatex,
    Xelatex,
    Lualatex,
}

impl Format {
    pub fn executable(self) -> &'static str {
        match self {
            Self::Latex => "latex",
            Self::Pdflatex => "pdflatex",
            Self::Xelatex => "xelatex",
            Self::Lualatex => "lualatex",
        }
    }
}

#[derive(Debug)]
pub struct Artifacts {
    pub directory: TempDir,
    pub log: String,
}

#[derive(Debug)]
pub enum CompileError {
    IO(io::Error),
    NotInstalled,
    Timeout(Elapsed),
}

impl From<io::Error> for CompileError {
    fn from(why: io::Error) -> Self {
        Self::IO(why)
    }
}

impl From<Elapsed> for CompileError {
    fn from(why: Elapsed) -> Self {
        Self::Timeout(why)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(why) => write!(f, "{}", why),
            Self::NotInstalled => write!(f, "TeX compiler not installed"),
            Self::Timeout(why) => write!(f, "{}", why),
        }
    }
}

impl error::Error for CompileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IO(why) => why.source(),
            Self::NotInstalled => None,
            Self::Timeout(why) => why.source(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CompileParams<'a> {
    pub format: Format,
    pub file_name: &'a str,
    pub code: &'a str,
    pub timeout: Duration,
}

impl<'a> Default for CompileParams<'a> {
    fn default() -> Self {
        Self {
            format: Format::Lualatex,
            file_name: "code.tex",
            code: "",
            timeout: Duration::from_secs(15),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Compiler<'a> {
    pub executable: &'a str,
    pub args: &'a [&'a str],
    pub file_name: &'a str,
    pub timeout: Duration,
}

impl<'a> Compiler<'a> {
    pub async fn compile<'b>(&'a self, code: &'b str) -> Result<Artifacts, CompileError> {
        let directory = tempdir()?;
        let tex_file = directory.path().join(self.file_name);
        fs::write(&tex_file, code).await?;

        let child = Command::new(self.executable)
            .args(self.args)
            .current_dir(&directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        timeout(self.timeout, child)
            .await?
            .map_err(|_| CompileError::NotInstalled)?;

        let log_file = tex_file.with_extension("log");
        let log_bytes = fs::read(log_file).await?;
        let log = String::from_utf8_lossy(&log_bytes).into_owned();
        Ok(Artifacts { directory, log })
    }
}
