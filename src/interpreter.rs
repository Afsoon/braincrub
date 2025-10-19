use core::ascii;

use crate::{
    io::{InputValue, OutputValue, ProgramValue},
    parser::{BrainfuckNodeAST, BrainfuckOperations},
};

pub struct Interpreter<'a, Display, Input>
where
    Display: OutputValue,
    Input: InputValue,
{
    pub memory_tape: Vec<u8>,
    pub memory_pointer: usize,
    pub ast_program: Option<&'a Vec<BrainfuckNodeAST>>,
    pub program_counter: Option<BrainfuckOperations>,
    pub display: Display,
    pub input: Input,
}

#[derive(Debug, PartialEq)]
pub enum InterpreterErrors {
    EmptyAST,
    UnknownASTNode,
}

impl<'a, Display, Input> Interpreter<'a, Display, Input>
where
    Display: OutputValue,
    Input: InputValue,
{
    pub fn new(display: Display, input: Input) -> Self {
        Interpreter {
            memory_tape: vec![0; 3000],
            memory_pointer: 0,
            ast_program: None,
            program_counter: None,
            display,
            input,
        }
    }

    pub fn load_ast_program(&mut self, ast_program: &'a Vec<BrainfuckNodeAST>) {
        self.ast_program = Some(ast_program);
    }

    pub fn run(&mut self) -> Result<(), InterpreterErrors> {
        let ast = match self.ast_program {
            Some(ast) if ast.len() == 0 => {
                return Err(InterpreterErrors::EmptyAST);
            }
            Some(ast) => ast,
            None => {
                return Err(InterpreterErrors::EmptyAST);
            }
        };

        let mut position: usize = 0;

        while let Some(node) = ast.get(position) {
            match node {
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::IncrementByOneCurrentCell =>
                {
                    position = command.next_position;
                    self.memory_tape[self.memory_pointer] += 1;
                    self.program_counter = Some(BrainfuckOperations::IncrementByOneCurrentCell)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::DecrementByOneCurrentCell =>
                {
                    position = command.next_position;
                    self.memory_tape[self.memory_pointer] -= 1;
                    self.program_counter = Some(BrainfuckOperations::DecrementByOneCurrentCell)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::MovePointerRight =>
                {
                    position = command.next_position;
                    self.memory_pointer += 1;
                    self.program_counter = Some(BrainfuckOperations::MovePointerRight)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::MovePointerLeft =>
                {
                    position = command.next_position;
                    self.memory_pointer -= 1;
                    self.program_counter = Some(BrainfuckOperations::MovePointerLeft)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::OutputCommand =>
                {
                    position = command.next_position;
                    let tape_value = self.memory_tape[self.memory_pointer];
                    match ascii::Char::from_u8(tape_value) {
                        Some(character) => {
                            self.display.print(ProgramValue::new(character.to_char()))
                        }
                        None => {
                            println!(
                                "Not valid ascii value, the current value is {:?}",
                                tape_value
                            )
                        }
                    }
                    self.program_counter = Some(BrainfuckOperations::OutputCommand)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::InputCommand =>
                {
                    position = command.next_position;
                    let input_value = self.input.get_input();
                    match input_value {
                        Ok(value) => {
                            self.memory_tape[self.memory_pointer] = value.into();
                        }
                        Err(_) => {
                            println!("Unable to read the input")
                        }
                    }
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::LoopEnd =>
                {
                    position = command.next_position;
                }
                BrainfuckNodeAST::Loop(loop_node)
                    if loop_node.operation == BrainfuckOperations::LoopStart =>
                {
                    let tape_value = self.memory_tape[self.memory_pointer];
                    if tape_value > 0 {
                        position = loop_node.next_position_as_true;
                        continue;
                    }
                    position = loop_node.next_position_as_false;
                }
                _ => return Err(InterpreterErrors::UnknownASTNode),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq)]
struct DebugMemoryPosition {
    position: usize,
    raw_value: u8,
    ascii_value: Option<char>,
}

#[cfg(test)]
impl<'a, Display, Input> Interpreter<'a, Display, Input>
where
    Display: OutputValue,
    Input: InputValue,
{
    fn get_debug_info_current_position(&self) -> DebugMemoryPosition {
        DebugMemoryPosition {
            position: self.memory_pointer,
            raw_value: self.memory_tape[self.memory_pointer],
            ascii_value: ascii::Char::from_u8(self.memory_tape[self.memory_pointer])
                .map(|charecter| charecter.to_char()),
        }
    }
}

#[cfg(test)]
mod interpreter_test {
    use std::iter::repeat_n;

    use crate::parser::BrainfuckASTBuilder;

    use super::*;

    #[derive(Debug, Copy, Clone)]
    struct NoRender;

    impl OutputValue for NoRender {
        fn print(&self, _value: ProgramValue) {
            ();
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct NoInput;

    impl InputValue for NoInput {
        fn get_input(&self) -> Result<ProgramValue, crate::io::InputError> {
            panic!("No input expect for this test")
        }
    }

    #[test]
    fn given_an_ast_empty_when_interpreter_is_run_then_return_error() {
        let mut interpeter = Interpreter::new(NoRender, NoInput);
        let builder = BrainfuckASTBuilder::new();

        interpeter.load_ast_program(&builder.ast);

        let error = interpeter.run().unwrap_err();

        assert_eq!(error, InterpreterErrors::EmptyAST)
    }

    #[test]
    fn give_an_ast_that_output_a_ascii_code_when_interpreter_is_run_then_display_a_ascii_value() {
        let mut interpeter = Interpreter::new(NoRender, NoInput);
        let mut builder = BrainfuckASTBuilder::new();
        let mut position: usize = 0;

        repeat_n(0, 65).for_each(|_value| {
            position += 1;
            builder.add_command_node(BrainfuckOperations::IncrementByOneCurrentCell, position);
        });

        builder.add_command_node(BrainfuckOperations::OutputCommand, 67);

        interpeter.load_ast_program(builder.build());

        let result = interpeter.run();

        let debug_expect = DebugMemoryPosition {
            position: 0,
            raw_value: 65,
            ascii_value: Some('A'),
        };

        assert!(result.is_ok());
        assert_eq!(interpeter.get_debug_info_current_position(), debug_expect)
    }

    #[test]
    fn given_an_ast_that_move_one_to_the_right_when_interpreter_is_run_then_the_current_position_is_1()
     {
        let mut interpeter = Interpreter::new(NoRender, NoInput);
        let mut builder = BrainfuckASTBuilder::new();

        let ast = builder
            .add_command_node(BrainfuckOperations::MovePointerRight, 1)
            .build();

        interpeter.load_ast_program(ast);

        let result = interpeter.run();

        let debug_expect = DebugMemoryPosition {
            position: 1,
            raw_value: 0,
            ascii_value: Some('\0'),
        };

        assert!(result.is_ok());
        assert_eq!(interpeter.get_debug_info_current_position(), debug_expect)
    }

    #[test]
    fn given_an_ast_that_move_two_to_the_right_and_one_to_left_when_interpreter_is_run_then_the_current_position_is_1()
     {
        let mut interpeter = Interpreter::new(NoRender, NoInput);
        let mut builder = BrainfuckASTBuilder::new();

        let ast = builder
            .add_command_node(BrainfuckOperations::MovePointerRight, 1)
            .add_command_node(BrainfuckOperations::MovePointerRight, 2)
            .add_command_node(BrainfuckOperations::MovePointerLeft, 3)
            .build();

        interpeter.load_ast_program(ast);

        let result = interpeter.run();

        let debug_expect = DebugMemoryPosition {
            position: 1,
            raw_value: 0,
            ascii_value: Some('\0'),
        };

        assert!(result.is_ok());
        assert_eq!(interpeter.get_debug_info_current_position(), debug_expect)
    }

    #[test]
    fn given_an_ast_with_input_command_when_interpreter_is_run_then_the_current_position_is_modified_with_the_value_provided()
     {
        #[derive(Debug, Copy, Clone)]
        struct AutomaticInput;

        impl InputValue for AutomaticInput {
            fn get_input(&self) -> Result<ProgramValue, crate::io::InputError> {
                Ok(ProgramValue('B'))
            }
        }

        let mut interpeter = Interpreter::new(NoRender, AutomaticInput);

        let mut builder = BrainfuckASTBuilder::new();

        let ast = builder
            .add_command_node(BrainfuckOperations::InputCommand, 1)
            .build();

        interpeter.load_ast_program(ast);

        let result = interpeter.run();

        let debug_expect = DebugMemoryPosition {
            position: 0,
            raw_value: 66,
            ascii_value: Some('B'),
        };

        assert!(result.is_ok());
        assert_eq!(interpeter.get_debug_info_current_position(), debug_expect)
    }

    #[test]
    fn given_an_ast_with_loops_to_render_a_uppercase_when_is_run_then_a_uppercase_is_show() {
        let mut interpeter = Interpreter::new(NoRender, NoInput);
        let mut builder = BrainfuckASTBuilder::new();
        builder
            .add_n_command_nodes(BrainfuckOperations::IncrementByOneCurrentCell, 10)
            .add_loop_node(BrainfuckOperations::LoopStart, 11, 21)
            .add_command_node(BrainfuckOperations::MovePointerRight, 12)
            .add_n_command_nodes(BrainfuckOperations::IncrementByOneCurrentCell, 6)
            .add_command_node(BrainfuckOperations::MovePointerLeft, 19)
            .add_command_node(BrainfuckOperations::DecrementByOneCurrentCell, 20)
            .add_command_node(BrainfuckOperations::LoopEnd, 10)
            .add_command_node(BrainfuckOperations::MovePointerRight, 22)
            .add_n_command_nodes(BrainfuckOperations::IncrementByOneCurrentCell, 5)
            .add_command_node(BrainfuckOperations::OutputCommand, 28);

        interpeter.load_ast_program(builder.build());

        let result = interpeter.run();

        let debug_expect = DebugMemoryPosition {
            position: 1,
            raw_value: 65,
            ascii_value: Some('A'),
        };

        assert!(result.is_ok());
        assert_eq!(interpeter.get_debug_info_current_position(), debug_expect)
    }
}
