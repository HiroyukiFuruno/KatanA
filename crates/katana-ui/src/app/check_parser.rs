use pulldown_cmark::{Options, Parser};

fn main() {
    let content = "[KatanA](https://github.com/KatanA)";
    let mut options = Options::empty();
    let parser = Parser::new_ext(content, options).into_offset_iter();

    for (event, range) in parser {
        println!("{:?}: {:?}", event, &content[range]);
    }
}
