use bibutils_sys::*;
use std::ffi::CString;
use std::fs;
use std::mem::MaybeUninit;
use tempfile::tempdir;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

pub unsafe fn convert(input: String, from: InputFormat, to: OutputFormat) -> String {
    let program = CString::new("texlab").unwrap();
    let mut context = MaybeUninit::zeroed();
    bibl_init(context.as_mut_ptr());
    let mut params = MaybeUninit::zeroed();
    bibl_initparams(
        params.as_mut_ptr(),
        from.read_mode() as i32,
        to.write_mode() as i32,
        program.as_ptr(),
    );

    let dir = tempdir().expect("failed to create a temporary directory");
    let input_path = dir.path().join("input");
    let input_path_ffi = CString::new(input_path.to_str().unwrap()).unwrap();
    fs::write(input_path, input).unwrap();
    let input_mode_ffi = CString::new("r").unwrap();

    let input_file = fopen(input_path_ffi.as_ptr(), input_mode_ffi.as_ptr());
    bibl_read(
        context.as_mut_ptr(),
        input_file,
        input_path_ffi.as_ptr(),
        params.as_mut_ptr(),
    );
    fclose(input_file);

    let output_path = dir.path().join("output");
    let output_path_ffi = CString::new(output_path.to_str().unwrap()).unwrap();
    let output_mode_ffi = CString::new("w").unwrap();
    let output_file = fopen(output_path_ffi.as_ptr(), output_mode_ffi.as_ptr());
    bibl_write(context.as_mut_ptr(), output_file, params.as_mut_ptr());
    fclose(output_file);
    bibl_freeparams(params.as_mut_ptr());
    bibl_free(context.as_mut_ptr());
    let data = fs::read(output_path).unwrap();
    // Remove BOM
    String::from_utf8_lossy(&data[3..]).into_owned()
}
