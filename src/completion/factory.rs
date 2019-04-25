use lsp_types::{CompletionItem, CompletionItemKind};

pub const KERNEL_DETAIL: &'static str = "built-in";

fn get_detail(component: Option<String>) -> String {
    component.unwrap_or_else(|| KERNEL_DETAIL.to_owned())
}

pub fn create_command(name: String, component: Option<String>) -> CompletionItem {
    let mut item = CompletionItem::new_simple(name, get_detail(component));
    item.kind = Some(CompletionItemKind::Function);
    item
}
