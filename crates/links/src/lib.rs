use base_db::{DocumentLocation, FeatureParams};

mod include;

pub fn find_links(params: FeatureParams) -> Vec<DocumentLocation> {
    let mut results = Vec::new();
    include::find_links(params, &mut results);
    results
}

#[cfg(test)]
mod tests;
