use std::{ffi::CString, fs, mem::MaybeUninit, path::Path};

use bibutils_sys::{
    bibl, bibl_free, bibl_freeparams, bibl_init, bibl_initparams, bibl_read, bibl_write, fclose,
    fopen, param, BIBL_ADSABSOUT, BIBL_BIBLATEXIN, BIBL_BIBTEXIN, BIBL_BIBTEXOUT, BIBL_COPACIN,
    BIBL_EBIIN, BIBL_ENDNOTEIN, BIBL_ENDNOTEOUT, BIBL_ENDNOTEXMLIN, BIBL_ISIOUT, BIBL_MEDLINEIN,
    BIBL_MODSIN, BIBL_MODSOUT, BIBL_NBIBIN, BIBL_NBIBOUT, BIBL_OK, BIBL_RISIN, BIBL_RISOUT,
    BIBL_WORD2007OUT, BIBL_WORDIN, FILE,
};
use tempfile::tempdir;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(unused)]
pub enum InputFormat {
    Bibtex,
    Biblatex,
    Copac,
    Ebi,
    Endnote,
    EndnoteXml,
    Medline,
    Mods,
    Nbib,
    Ris,
    Word,
}

impl InputFormat {
    fn read_mode(self) -> u32 {
        match self {
            Self::Bibtex => BIBL_BIBTEXIN,
            Self::Biblatex => BIBL_BIBLATEXIN,
            Self::Copac => BIBL_COPACIN,
            Self::Ebi => BIBL_EBIIN,
            Self::Endnote => BIBL_ENDNOTEIN,
            Self::EndnoteXml => BIBL_ENDNOTEXMLIN,
            Self::Medline => BIBL_MEDLINEIN,
            Self::Mods => BIBL_MODSIN,
            Self::Nbib => BIBL_NBIBIN,
            Self::Ris => BIBL_RISIN,
            Self::Word => BIBL_WORDIN,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(unused)]
pub enum OutputFormat {
    Adsabs,
    Bibtex,
    Endnote,
    Isi,
    Mods,
    Nbib,
    Ris,
    Word2007,
}

impl OutputFormat {
    fn write_mode(self) -> u32 {
        match self {
            Self::Adsabs => BIBL_ADSABSOUT,
            Self::Bibtex => BIBL_BIBTEXOUT,
            Self::Endnote => BIBL_ENDNOTEOUT,
            Self::Isi => BIBL_ISIOUT,
            Self::Mods => BIBL_MODSOUT,
            Self::Nbib => BIBL_NBIBOUT,
            Self::Ris => BIBL_RISOUT,
            Self::Word2007 => BIBL_WORD2007OUT,
        }
    }
}

struct Context {
    inner: MaybeUninit<bibl>,
}

impl Context {
    fn new() -> Self {
        let mut inner = MaybeUninit::zeroed();
        unsafe {
            bibl_init(inner.as_mut_ptr());
        }
        Self { inner }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            bibl_free(self.inner.as_mut_ptr());
        }
    }
}

unsafe impl Send for Context {}

struct Params {
    inner: MaybeUninit<param>,
}

impl Params {
    fn new(from: InputFormat, to: OutputFormat) -> Self {
        let program = CString::new("texlab").unwrap();
        let mut inner = MaybeUninit::zeroed();
        unsafe {
            bibl_initparams(
                inner.as_mut_ptr(),
                from.read_mode() as i32,
                to.write_mode() as i32,
                program.as_ptr() as *mut std::os::raw::c_char,
            );
        }
        Self { inner }
    }
}

impl Drop for Params {
    fn drop(&mut self) {
        unsafe {
            bibl_freeparams(self.inner.as_mut_ptr());
        }
    }
}

unsafe impl Send for Params {}

struct File {
    path: CString,
    handle: *mut FILE,
}

impl File {
    fn new<M: Into<Vec<u8>>>(path: &Path, mode: M) -> Self {
        let path = CString::new(path.to_str().unwrap()).unwrap();
        let mode = CString::new(mode).unwrap();
        let handle = unsafe { fopen(path.as_ptr(), mode.as_ptr()) };
        Self { path, handle }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            fclose(self.handle);
        }
    }
}

unsafe impl Send for File {}

pub fn convert(input: &str, from: InputFormat, to: OutputFormat) -> Option<String> {
    let mut context = Context::new();
    let mut params = Params::new(from, to);
    let dir = tempdir().expect("failed to create a temporary directory");

    let input_path = dir.path().join("input");
    fs::write(&input_path, input).ok()?;
    let input_file = File::new(&input_path, "r");
    unsafe {
        let status = bibl_read(
            context.inner.as_mut_ptr(),
            input_file.handle,
            input_file.path.as_ptr() as *mut std::os::raw::c_char,
            params.inner.as_mut_ptr(),
        );

        if status != BIBL_OK as i32 {
            return None;
        }
    }

    let output_path = dir.path().join("output");
    let output_file = File::new(&output_path, "w");
    unsafe {
        let status = bibl_write(
            context.inner.as_mut_ptr(),
            output_file.handle,
            params.inner.as_mut_ptr(),
        );

        if status != BIBL_OK as i32 {
            return None;
        }
    }

    // Remove BOM
    let data = fs::read(&output_path).ok()?;
    if data.is_empty() {
        return None;
    }

    let text = String::from_utf8_lossy(&data[3..]).into_owned();
    Some(text)
}
