#![feature(ascii_char)]
/**
 * The previous line is mandatory to be able to use the experimental ascii handle api
 */
pub mod file;
pub mod interpreter;
pub mod io;
pub mod parser;

use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

use crate::file::read_source_code_file;
use crate::interpreter::{Interpreter, InterpreterConfig};
use crate::io::{BasicInput, BasicOutput, BrainfuckMemory, MemoryTape};
use crate::parser::from_source_to_node_ast;

pub fn path_parser(path_string: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(path_string))
}

pub fn memory_size_parser(memory_size: &str) -> Result<usize, String> {
    match memory_size.to_owned().parse::<usize>() {
        Ok(value) if value > 30_000 => Err("Maximum value accepted is 30_000".to_string()),
        Ok(value) if value < 1 => Err("Minimum value accepted is 1".to_string()),
        Ok(value) => Ok(value),
        Err(err) => Err(err.to_string()),
    }
}

pub fn limit_read_instructions_parser(limit_read_instructions: &str) -> Result<usize, String> {
    match limit_read_instructions.to_owned().parse::<usize>() {
        Ok(value) if value > 100_000 => Err("Maximum value accepted is 100_000".to_string()),
        Ok(value) if value < 1 => Err("Minimum value accepted is 1".to_string()),
        Ok(value) => Ok(value),
        Err(err) => Err(err.to_string()),
    }
}

fn braincrub_cli() -> Command {
    Command::new("braincrub")
        .about("A Brainfuck interperter to lint, run brainfuck source code files.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("lint")
                .about("Check if the file provided have correct brainfuck syntax. It will fail if the loops aren't balanced. Non valid characters are ignored")
                .arg(
                    Arg::new("file")
                        .action(ArgAction::Set)
                        .value_name("PATH")
                        .help("File path to the file to be processed")
                        .num_args(1)
                        .value_parser(path_parser)
                        .required(true)
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("run")
                .about("Check and run a brainfuck source code file. Non valid characters are ignored")
                .arg(
                    Arg::new("memory-size")
                        .action(ArgAction::Set)
                        .required(false)
                        .num_args(1)
                        .default_value("3000")
                        .help("Size of the vec to simulate the memory to save the data. The maximum size is 30_000 memory cells")
                        .value_parser(memory_size_parser)
                )
                .arg(
                    Arg::new("limit-read-instructions")
                        .short('l')
                        .action(ArgAction::Set)
                        .num_args(1)
                        .default_value("60000")
                        .value_parser(limit_read_instructions_parser)
                        .help("Number of instructions the cli can process before to consider we are on a infinite loop")
                        .required(false)
                )
                .arg(
                    Arg::new("file")
                        .short('f')
                        .action(ArgAction::Set)
                        .value_name("PATH")
                        .help("Path to the file to be processed")
                        .num_args(1)
                        .value_parser(path_parser)
                        .required(true)
                )
                .arg_required_else_help(true),
        )
}

fn main() {
    let matches = braincrub_cli().get_matches();

    match matches.subcommand() {
        Some(("lint", sub_matches)) => {
            let path = sub_matches
                .get_one::<PathBuf>("file")
                .unwrap()
                .to_str()
                .expect("Expected a valid path string as it was parsed before");

            let source_code = read_source_code_file(path)
                .map_err(|error| panic!("{:?}", error.to_string()))
                .unwrap();

            from_source_to_node_ast(&source_code)
                .map_err(|error| panic!("{:?}", error.to_string()))
                .unwrap();

            println!("All good!");
        }
        Some(("run", sub_matches)) => {
            let path = sub_matches
                .get_one::<PathBuf>("file")
                .unwrap()
                .to_str()
                .expect("Expected a valid path string as it was parsed before");

            let memory_tape_size = sub_matches
                .get_one::<usize>("memory-size")
                .expect("Expecte a valid memory tape size");

            let limit_read_instructions = sub_matches
                .get_one::<usize>("limit-read-instructions")
                .unwrap();

            let source_code = read_source_code_file(path)
                .map_err(|error| panic!("{:?}", error.to_string()))
                .unwrap();

            let ast = from_source_to_node_ast(&source_code)
                .map_err(|error| panic!("{:?}", error.to_string()))
                .unwrap();

            let mut interpreter = Interpreter::new(
                BasicOutput,
                BasicInput::default(),
                BrainfuckMemory::new(*memory_tape_size),
                InterpreterConfig::new(*limit_read_instructions),
            );

            interpreter.load_ast_program(&ast);

            interpreter
                .run()
                .map_err(|error| panic!("{:?}", error.to_string()))
                .unwrap();

            println!("");
            println!("Program executed succesfully");
        }
        _ => {
            panic!("command doesn't exist")
        }
    }
}
