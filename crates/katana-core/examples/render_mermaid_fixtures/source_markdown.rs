pub fn extract_mermaid_block(markdown: &str) -> String {
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in markdown.lines() {
        if matches!(line.trim(), "```mermaid" | "~~~mermaid") {
            in_block = true;
            continue;
        }
        if in_block && matches!(line.trim(), "```" | "~~~") {
            return lines.join("\n");
        }
        if in_block {
            lines.push(line);
        }
    }
    panic!("Mermaid block not found");
}
