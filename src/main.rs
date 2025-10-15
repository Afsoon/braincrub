pub mod file;
pub mod parser;

use crate::file::read_source_code_file;
use crate::parser::from_source_to_ast;

fn main() {
    let source_code_raw = read_source_code_file("./test_output_a.txt");

    if let Err(error) = source_code_raw {
        eprintln!("{}", error);
        return;
    }

    let ast = from_source_to_ast(&source_code_raw.unwrap());
    println!("The AST is {:?}", ast)
}
