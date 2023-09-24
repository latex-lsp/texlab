mod builder;
pub mod matchers;
mod patterns;

pub use builder::*;
pub use patterns::*;

pub fn included_packages<'a>(
    params: &'a base_db::FeatureParams<'a>,
) -> impl Iterator<Item = &completion_data::Package<'_>> + 'a {
    let db = &completion_data::DATABASE;
    let documents = params.project.documents.iter();
    let links = documents
        .filter_map(|document| document.data.as_tex())
        .flat_map(|data| data.semantics.links.iter());

    links
        .filter_map(|link| link.package_name())
        .filter_map(|name| db.find(&name))
        .chain(std::iter::once(db.kernel()))
        .flat_map(|pkg| {
            pkg.references
                .iter()
                .filter_map(|name| db.find(name))
                .chain(std::iter::once(pkg))
        })
}

pub struct ProviderContext<'a, 'b> {
    pub builder: &'b mut CompletionBuilder<'a>,
    pub params: &'a crate::CompletionParams<'a>,
    pub cursor: base_db::semantics::Span,
}
