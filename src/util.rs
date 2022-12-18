pub mod capabilities;
pub mod chktex;
pub mod components;
pub mod cursor;
pub mod label;
pub mod lang_data;
pub mod line_index;
pub mod line_index_ext;
pub mod lsp_enums;

use std::path::PathBuf;

use once_cell::sync::Lazy;

pub static HOME_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| dirs::home_dir());
