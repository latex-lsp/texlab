use texlab::latex;

fn main() {
    let green_node = latex::parse(include_str!(
        "D:/uni/bachelor-thesis/ausarbeitung/thesis.tex"
    ))
    .green_node;
    let root = latex::SyntaxNode::new_root(green_node);
    println!("{:#?}", root);
}
