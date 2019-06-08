use super::ini;
use super::*;
use std::fs;
use std::path::Path;

pub const DATABASE_PATH: &str = "miktex/config/package-manifests.ini";
const INSTALLED_PATH: &str = "miktex/config/packages.ini";

pub fn read_database(file: &Path, root_dir: &Path) -> Option<Vec<PackageManifest>> {
    let manifests = fs::read_to_string(&file).ok()?;
    let (_, manifests) = ini::parser::parse(&manifests).ok()?;

    let installed = fs::read_to_string(&root_dir.join(INSTALLED_PATH)).ok()?;
    let (_, installed) = ini::parser::parse(&installed).ok()?;

    let packages = manifests
        .sections
        .into_iter()
        .filter(|x| !x.name.starts_with('_'))
        .flat_map(|x| read_manifest(&x, &installed, root_dir))
        .collect();

    Some(packages)
}

fn read_manifest(
    section: &ini::Section,
    installed: &ini::Ini,
    root_dir: &Path,
) -> Option<PackageManifest> {
    let name = section.name.to_owned();
    let title = section.get_string_value("title")?.to_owned();
    let description = section
        .get_array_value("description")
        .and_then(|x| x.first())
        .map(|x| x.to_string());
    let doc_files = section
        .get_array_value("doc")?
        .iter()
        .map(|x| root_dir.join(x))
        .collect();
    let run_files = section
        .get_array_value("run")?
        .iter()
        .map(|x| root_dir.join(x))
        .collect();
    let is_installed = installed.sections.iter().any(|x| x.name == name);

    Some(PackageManifest {
        name,
        title,
        description,
        doc_files,
        run_files,
        is_installed,
    })
}
