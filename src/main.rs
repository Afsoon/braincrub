#![feature(ascii_char)]
/**
 * The previous line is mandatory to be able to use the experimental ascii handle api
 */
pub mod file;
pub mod interpreter;
pub mod io;
pub mod parser;

use crate::file::read_source_code_file;
use crate::interpreter::Interpreter;
use crate::io::{BasicInput, BasicOutput};
use crate::parser::from_source_to_node_ast;

fn main() {
    let source_code_raw = read_source_code_file("./test_hello_world.txt");

    if let Err(error) = source_code_raw {
        eprintln!("{}", error);
        return;
    }

    let ast = from_source_to_node_ast(&source_code_raw.unwrap()).unwrap();

    let mut interpreter = Interpreter::new(BasicOutput, BasicInput::default());
    interpreter.load_ast_program(&ast);

    interpreter.run().unwrap();
}
