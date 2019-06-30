use super::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub const DATABASE_PATH: &str = "miktex/data/le";
const FNDB_SIGNATURE: u32 = 0x42444e46;
const FNDB_WORD_SIZE: usize = 4;
const FNDB_TABLE_POINTER_OFFSET: usize = 4 * FNDB_WORD_SIZE;
const FNDB_TABLE_SIZE_OFFSET: usize = 6 * FNDB_WORD_SIZE;
const FNDB_ENTRY_SIZE: usize = 4 * FNDB_WORD_SIZE;

pub fn read_database(directory: &Path) -> Result<Vec<PathBuf>> {
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
        database.extend(parse_database(&bytes).map_err(|_| Error::CorruptFileDatabase)?);
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
