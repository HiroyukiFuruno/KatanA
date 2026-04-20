fn main() {
    let md = "### heading\n\nKatanA rendering pipeline:\n\n```mermaid\ngraph LR\n```\n# next";
    let close_idx = md.find("\n```").unwrap();
    println!("close_idx: {}", close_idx);
    let after_start = close_idx + "\n```".len();
    let after = &md[after_start..];
    println!("after: {:?}", after);
    
    let after_stripped = after.strip_prefix('\n').unwrap_or(after);
    println!("after_stripped: {:?}", after_stripped);
}
