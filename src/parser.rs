#[cfg(test)]
use std::iter::repeat_n;

use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BrainfuckOperations {
    MovePointerRight,
    MovePointerLeft,
    IncrementByOneCurrentCell,
    DecrementByOneCurrentCell,
    InputCommand,
    OutputCommand,
    LoopStart,
    LoopEnd,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CommandInformation {
    pub operation: BrainfuckOperations,
    pub next_position: usize, // Change to Option
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LoopInformation {
    pub operation: BrainfuckOperations,
    pub next_position_as_true: usize,  // Change to Option
    pub next_position_as_false: usize, // Change to Option
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BrainfuckNodeAST {
    Command(CommandInformation),
    Loop(LoopInformation),
    NoOp,
}

#[cfg(test)]
pub struct BrainfuckASTBuilder {
    pub ast: Vec<BrainfuckNodeAST>,
}

#[cfg(test)]
impl BrainfuckASTBuilder {
    pub fn new() -> Self {
        BrainfuckASTBuilder { ast: vec![] }
    }

    pub fn add_command_node(
        &mut self,
        operation: BrainfuckOperations,
        next_position: usize,
    ) -> &mut Self {
        self.ast.push(BrainfuckNodeAST::Command(CommandInformation {
            operation,
            next_position,
        }));
        self
    }

    pub fn add_n_command_nodes(
        &mut self,
        operation: BrainfuckOperations,
        n_times: usize,
    ) -> &mut Self {
        repeat_n(0, n_times).for_each(|_value| {
            self.ast.push(BrainfuckNodeAST::Command(CommandInformation {
                operation,
                next_position: self.ast.len() + 1,
            }))
        });
        self
    }

    pub fn add_loop_node(
        &mut self,
        operation: BrainfuckOperations,
        next_position_as_true: usize,
        next_position_as_false: usize,
    ) -> &mut Self {
        self.ast.push(BrainfuckNodeAST::Loop(LoopInformation {
            operation,
            next_position_as_true,
            next_position_as_false,
        }));
        self
    }

    pub fn build(&self) -> &Vec<BrainfuckNodeAST> {
        &self.ast
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ParserErrors {
    #[error("The source code have more open loop brackets than closing loop brackets.")]
    MissingTerminantedLoop,
    #[error("The source code have more closing loop brackets than open loop brackets.")]
    MissingOpenLoop,
}

fn map_char_to_brainfuck_operation(token: char) -> Option<BrainfuckOperations> {
    match token {
        '>' => Some(BrainfuckOperations::MovePointerRight),
        '<' => Some(BrainfuckOperations::MovePointerLeft),
        '+' => Some(BrainfuckOperations::IncrementByOneCurrentCell),
        '-' => Some(BrainfuckOperations::DecrementByOneCurrentCell),
        ',' => Some(BrainfuckOperations::InputCommand),
        '.' => Some(BrainfuckOperations::OutputCommand),
        '[' => Some(BrainfuckOperations::LoopStart),
        ']' => Some(BrainfuckOperations::LoopEnd),
        _ => None,
    }
}

pub fn from_source_to_node_ast(source_code: &str) -> Result<Vec<BrainfuckNodeAST>, ParserErrors> {
    let mut loop_start_position: Vec<usize> = vec![];
    let mut program_ast_vec: Vec<BrainfuckNodeAST> = vec![];
    let mut iter_chars = source_code.chars().into_iter();

    while let Some(token) = iter_chars.next() {
        match map_char_to_brainfuck_operation(token) {
            Some(BrainfuckOperations::LoopStart) => {
                loop_start_position.push(program_ast_vec.len());
                program_ast_vec.push(BrainfuckNodeAST::Command(CommandInformation {
                    operation: BrainfuckOperations::LoopStart,
                    next_position: program_ast_vec.len() + 1,
                }));
            }
            Some(BrainfuckOperations::LoopEnd) => match loop_start_position.pop() {
                Some(last_position_recorded) => {
                    program_ast_vec.push(BrainfuckNodeAST::Command(CommandInformation {
                        operation: BrainfuckOperations::LoopEnd,
                        next_position: last_position_recorded,
                    }));
                    program_ast_vec[last_position_recorded] =
                        BrainfuckNodeAST::Loop(LoopInformation {
                            operation: BrainfuckOperations::LoopStart,
                            next_position_as_true: last_position_recorded + 1,
                            next_position_as_false: program_ast_vec.len(),
                        })
                }
                None => return Err(ParserErrors::MissingOpenLoop),
            },
            Some(value) => {
                program_ast_vec.push(BrainfuckNodeAST::Command(CommandInformation {
                    operation: value,
                    next_position: program_ast_vec.len() + 1,
                }));
            }
            None => (),
        }
    }

    if loop_start_position.len() > 0 {
        return Err(ParserErrors::MissingTerminantedLoop);
    }

    Ok(program_ast_vec)
}

#[cfg(test)]
mod parser_source_code_test {
    use super::*;

    #[test]
    fn given_a_source_code_with_some_invalid_characters_when_parse_it_to_ast_then_return_an_ast_with_invalid_characters_filtered()
     {
        let input = "b[de+a-]";
        let mut builder = BrainfuckASTBuilder::new();

        let result = from_source_to_node_ast(input)
            .expect("The input should return only parsed the valid characters, the rest ignore it");

        assert_eq!(
            result,
            *builder
                .add_loop_node(BrainfuckOperations::LoopStart, 1, 4)
                .add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, 2)
                .add_command_node(BrainfuckOperations::DecrementByOneCurrentCell, 3)
                .add_command_node(BrainfuckOperations::LoopEnd, 0)
                .build(),
        )
    }

    #[test]
    fn given_a_source_code_with_only_invalid_characters_when_parse_it_to_ast_then_return_an_empty_array()
     {
        let input = "not a none valid character!!!";

        let result = from_source_to_node_ast(input)
            .expect("Expected an empty array as we ignore everything not related to brainfuck");

        assert_eq!(result, [])
    }

    #[test]
    fn given_a_loop_ast_formatted_then_return_a_correct_ast_with_their_information() {
        let input = "++[+>]++";
        let mut builder = BrainfuckASTBuilder::new();

        let result = from_source_to_node_ast(input)
            .expect("Expected a vec with the   dtoken correctly parsed to their AST");

        assert_eq!(
            result,
            *builder
                .add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, 1)
                .add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, 2)
                .add_loop_node(BrainfuckOperations::LoopStart, 3, 6)
                .add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, 4)
                .add_command_node(BrainfuckOperations::MovePointerRight, 5)
                .add_command_node(BrainfuckOperations::LoopEnd, 2)
                .add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, 7)
                .add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, 8)
                .build()
        )
    }

    #[test]
    fn given_a_terminated_loop_found_before_an_open_loop_then_return_an_error() {
        let input = "+]";

        let result = from_source_to_node_ast(input)
            .expect_err("Expected an error as the first bracked found is close loop");

        assert_eq!(result, ParserErrors::MissingOpenLoop)
    }

    #[test]
    fn given_double_terminated_loop_found_after_one_open_loop_then_return_an_error() {
        let input = "+++[+++++]---]";

        let result = from_source_to_node_ast(input)
            .expect_err("Expected an error as we have unbalanced brackets in the loop");

        assert_eq!(result, ParserErrors::MissingOpenLoop)
    }

    #[test]
    fn given_a_open_bracket_without_his_matched_bracket_then_return_an_error() {
        let input = "[+++";

        let result = from_source_to_node_ast(input).expect_err(
            "Expected an error as the parsed ended before could find a matched bracket",
        );

        assert_eq!(result, ParserErrors::MissingTerminantedLoop)
    }
}
