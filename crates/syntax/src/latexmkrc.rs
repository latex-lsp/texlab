use std::sync::Arc;

use distro::FileNameDB;

#[derive(Debug, Clone, Default)]
pub struct LatexmkrcData {
    pub aux_dir: Option<String>,
    pub out_dir: Option<String>,
    pub file_name_db: Arc<FileNameDB>,
}
