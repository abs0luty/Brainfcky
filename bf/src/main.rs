use std::{env::args, fs, process::exit};

use codegen::Codegen;
use parser::Parser;

mod codegen;
mod parser;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("usage: bf <filename>");
        exit(1);
    }

    let filename = args.get(1).unwrap();

    match fs::read_to_string(filename) {
        Ok(ref contents) => unsafe {
            println!("Building started...");
            let mut codegen = Codegen::new(filename, Parser::new(contents));
            codegen.build();
        },
        Err(_) => {
            eprintln!("Error occured when reading file");
            exit(1);
        }
    }
}
