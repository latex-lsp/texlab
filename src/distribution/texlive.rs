use super::tlpdb;
use super::*;
use std::fs;
use std::path::Path;

pub const DATABASE_PATH: &str = "../tlpkg/texlive.tlpdb";

pub fn read_database(file: &Path, root_dir: &Path) -> Option<Vec<PackageManifest>> {
    let database = fs::read_to_string(&file).ok()?;
    let (_, database) = tlpdb::parser::parse(&database).ok()?;
    let packages = database
        .sections
        .into_iter()
        .filter(|x| !x.name.starts_with("00"))
        .flat_map(|x| read_manifest(&x, root_dir))
        .collect();

    Some(packages)
}

fn read_manifest(section: &ini::Section, root_dir: &Path) -> Option<PackageManifest> {
    let name = section.name.to_owned();
    let title = section.get_string_value("shortdesc")?.to_owned();
    let description = section.get_array_value("longdesc").map(|x| x.join("\n"));
    let doc_files = section
        .get_array_value("docfiles")?
        .iter()
        .map(|x| root_dir.join(x))
        .collect();
    let run_files = section
        .get_array_value("runfiles")?
        .iter()
        .map(|x| root_dir.join(x))
        .collect();

    Some(PackageManifest {
        name,
        title,
        description,
        doc_files,
        run_files,
        is_installed: true,
    })
}
