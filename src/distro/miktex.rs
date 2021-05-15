use std::{
    ffi::OsStr,
    fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};

use super::kpsewhich::{self, Resolver};

pub fn load_resolver() -> Result<Resolver> {
    let root_directories = kpsewhich::root_directories()?;
    let resolver = kpsewhich::parse_database(&root_directories, read_database)?;
    Ok(resolver)
}

const DATABASE_PATH: &str = "miktex/data/le";
const FNDB_SIGNATURE: u32 = 0x42_44_4e_46;
const FNDB_WORD_SIZE: usize = 4;
const FNDB_TABLE_POINTER_OFFSET: usize = 4 * FNDB_WORD_SIZE;
const FNDB_TABLE_SIZE_OFFSET: usize = 6 * FNDB_WORD_SIZE;
const FNDB_ENTRY_SIZE: usize = 4 * FNDB_WORD_SIZE;

fn read_database(directory: &Path) -> Result<Vec<PathBuf>> {
    let database_directory = directory.join(DATABASE_PATH);
    if !database_directory.exists() {
        return Ok(Vec::new());
    }

    let mut database = Vec::new();
    for file in fs::read_dir(database_directory)?.filter_map(Result::ok) {
        if file.path().extension().and_then(OsStr::to_str) == Some("fndb-5") {
            let bytes = fs::read(file.path())?;
            database.extend(parse_database(&bytes).context("parsing kpsewhich database")?);
        }
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
