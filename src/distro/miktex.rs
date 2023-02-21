use std::{
    ffi::OsStr,
    fs,
    io::{self, Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

const DATABASE_PATH: &str = "miktex/data/le";
const FNDB_SIGNATURE: u32 = 0x42_44_4e_46;
const FNDB_WORD_SIZE: u32 = 4;
const FNDB_TABLE_POINTER_OFFSET: u32 = 4 * FNDB_WORD_SIZE;
const FNDB_TABLE_SIZE_OFFSET: u32 = 6 * FNDB_WORD_SIZE;
const FNDB_ENTRY_SIZE: u32 = 4 * FNDB_WORD_SIZE;

pub(super) fn read_database(directory: &Path) -> Result<Vec<PathBuf>> {
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
    if read_u32(&mut reader)? != FNDB_SIGNATURE {
        return Err(io::ErrorKind::InvalidData.into());
    }

    reader.set_position(u64::from(FNDB_TABLE_POINTER_OFFSET));
    let table_address = read_u32(&mut reader)?;

    reader.set_position(u64::from(FNDB_TABLE_SIZE_OFFSET));
    let table_size = read_u32(&mut reader)?;

    let mut files = Vec::new();
    for i in 0..table_size {
        let offset = table_address + i * FNDB_ENTRY_SIZE;
        reader.set_position(u64::from(offset));
        let file_name_offset = read_u32(&mut reader)? as usize;
        let directory_offset = read_u32(&mut reader)? as usize;
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

fn read_u32(reader: &mut Cursor<&[u8]>) -> io::Result<u32> {
    let mut buf = [0u8; std::mem::size_of::<u32>()];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}
