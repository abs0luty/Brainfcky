use codegen::Codegen;
use parser::Parser;

mod codegen;
mod parser;

fn main() {
    unsafe {
        println!("Building started...");
        let mut codegen = Codegen::new(Parser::new(",><."));
        codegen.build();
        println!("Finished!");
    }
}
