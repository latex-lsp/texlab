use base_db::{DocumentLocation, FeatureParams};

mod include;

pub fn find_links<'a>(params: &FeatureParams<'a>) -> Vec<DocumentLocation<'a>> {
    let mut results = Vec::new();
    include::find_links(params, &mut results);
    results
}

#[cfg(test)]
mod tests;
