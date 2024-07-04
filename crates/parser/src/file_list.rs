use syntax::file_list::FileList;

pub fn parse_file_list(input: &str) -> FileList {
    let mut file_list = FileList::default();
    for line in input.lines() {
        if let Some(working_dir) = line.strip_prefix("PWD ") {
            file_list.working_dir = Some(working_dir.into());
        } else if let Some(input) = line.strip_prefix("INPUT ") {
            file_list.inputs.insert(input.into());
        } else if let Some(output) = line.strip_prefix("OUTPUT ") {
            file_list.outputs.insert(output.into());
        }
    }

    file_list
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashSet;

    use super::*;

    #[test]
    fn test_parse_file_list() {
        let input = r#"
PWD /home/user
INPUT file1.tex
INPUT file1.tex
OUTPUT file2.pdf"#;

        let expected_file_list = FileList {
            working_dir: Some("/home/user".into()),
            inputs: FxHashSet::from_iter(["file1.tex".into()]),
            outputs: FxHashSet::from_iter(["file2.pdf".into()]),
        };
        let actual_file_list = parse_file_list(input);
        assert_eq!(actual_file_list, expected_file_list);

        assert_eq!(actual_file_list.working_dir, Some("/home/user".into()));
        assert_eq!(actual_file_list.inputs.len(), 1);
        assert_eq!(actual_file_list.outputs.len(), 1);
    }
}
