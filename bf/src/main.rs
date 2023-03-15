use codegen::Codegen;
use parser::Parser;

mod codegen;
mod parser;

fn main() {
    unsafe {
        let codegen = Codegen::new(Parser::new("test"));
        codegen.build();
    }
}
