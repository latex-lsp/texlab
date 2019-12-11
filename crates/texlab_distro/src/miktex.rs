use super::compile::*;
use super::kpsewhich;
use super::{Distribution, DistributionKind, LoadError, Resolver};
use byteorder::{LittleEndian, ReadBytesExt};
use futures::lock::Mutex;
use futures_boxed::boxed;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Cursor};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Miktex {
    resolver: Mutex<Arc<Resolver>>,
}

impl Miktex {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Distribution for Miktex {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Miktex
    }

    fn supports_format(&self, format: Format) -> bool {
        match format {
            Format::Latex | Format::Pdflatex => true,
            Format::Xelatex | Format::Lualatex => true,
        }
    }

    #[boxed]
    async fn load(&self) -> Result<(), LoadError> {
        let resolver = Arc::new(kpsewhich::parse_database(read_database).await?);
        let mut resolver_guard = self.resolver.lock().await;
        mem::replace(&mut *resolver_guard, resolver);
        Ok(())
    }

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver> {
        let resolver = self.resolver.lock().await;
        Arc::clone(&resolver)
    }
}

const DATABASE_PATH: &str = "miktex/data/le";
const FNDB_SIGNATURE: u32 = 0x42444e46;
const FNDB_WORD_SIZE: usize = 4;
const FNDB_TABLE_POINTER_OFFSET: usize = 4 * FNDB_WORD_SIZE;
const FNDB_TABLE_SIZE_OFFSET: usize = 6 * FNDB_WORD_SIZE;
const FNDB_ENTRY_SIZE: usize = 4 * FNDB_WORD_SIZE;

fn read_database(directory: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let database_directory = directory.join(DATABASE_PATH);
    if !database_directory.exists() {
        return Ok(Vec::new());
    }

    let mut database = Vec::new();
    let files = fs::read_dir(database_directory)
        .expect("Could not traverse database directory")
        .filter_map(|x| x.ok())
        .filter(|x| x.path().extension().and_then(OsStr::to_str) == Some("fndb-5"));

    for file in files {
        let bytes = fs::read(file.path()).expect("Could not read fndb file");
        database.extend(parse_database(&bytes).map_err(|_| LoadError::CorruptFileDatabase)?);
    }

    Ok(database)
}

fn parse_database(bytes: &[u8]) -> io::Result<Vec<PathBuf>> {
    let mut reader = Cursor::new(bytes);
    if reader.read_u32::<LittleEndian>()? != FNDB_SIGNATURE {
        return Err(io::ErrorKind::InvalidData.into());
    }

    reader.set_position(FNDB_TABLE_POINTER_OFFSET as u64);
    let table_address = reader.read_u32::<LittleEndian>()?;

    reader.set_position(FNDB_TABLE_SIZE_OFFSET as u64);
    let table_size = reader.read_u32::<LittleEndian>()?;

    let mut files = Vec::new();
    for i in 0..table_size {
        let offset = table_address + i * FNDB_ENTRY_SIZE as u32;
        reader.set_position(offset as u64);
        let file_name_offset = reader.read_u32::<LittleEndian>()? as usize;
        let directory_offset = reader.read_u32::<LittleEndian>()? as usize;
        let file_name = read_string(bytes, file_name_offset)?;
        let directory = read_string(bytes, directory_offset)?;

        let file = PathBuf::from(directory).join(file_name);
        files.push(file);
    }

    Ok(files)
}

fn read_string(bytes: &[u8], offset: usize) -> io::Result<&str> {
    let mut byte = bytes[offset];
    let mut length = 0;
    while byte != 0x00 {
        length += 1;
        byte = bytes[offset + length];
    }

    std::str::from_utf8(&bytes[offset..offset + length])
        .map_err(|_| io::ErrorKind::InvalidData.into())
}
