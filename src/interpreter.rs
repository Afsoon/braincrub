use core::ascii;

use crate::{
    io::{InputValue, OutputValue, ProgramValue},
    parser::BrainfuckOperations,
};

pub struct Interpreter<Display, Input>
where
    Display: OutputValue,
    Input: InputValue,
{
    pub memory_tape: Vec<u8>,
    pub memory_pointer: usize,
    pub ast_program: Vec<BrainfuckOperations>,
    pub program_counter: Option<BrainfuckOperations>,
    pub display: Display,
    pub input: Input,
}

#[derive(Debug, PartialEq)]
pub enum InterpreterErrors {
    EmptyAST,
}

impl<Display, Input> Interpreter<Display, Input>
where
    Display: OutputValue,
    Input: InputValue,
{
    pub fn new(display: Display, input: Input) -> Self {
        Interpreter {
            memory_tape: vec![0; 3000],
            memory_pointer: 0,
            ast_program: vec![],
            program_counter: None,
            display,
            input,
        }
    }

    pub fn load_ast_program(&mut self, ast_program: Vec<BrainfuckOperations>) {
        self.ast_program = ast_program;
    }

    pub fn run(&mut self) -> Result<(), InterpreterErrors> {
        if self.ast_program.len() == 0 {
            return Err(InterpreterErrors::EmptyAST);
        }

        let mut memory_tape_iterator = self.ast_program.iter();

        while let Some(instruction) = memory_tape_iterator.next() {
            match instruction {
                BrainfuckOperations::IncrementByOneCurrentCell => {
                    self.memory_tape[self.memory_pointer] += 1;
                    self.program_counter = Some(BrainfuckOperations::IncrementByOneCurrentCell)
                }
                BrainfuckOperations::DecrementByOneCurrentCell => {
                    self.memory_tape[self.memory_pointer] -= 1;
                    self.program_counter = Some(BrainfuckOperations::DecrementByOneCurrentCell)
                }
                BrainfuckOperations::MovePointerRight => {
                    self.memory_pointer += 1;
                    self.program_counter = Some(BrainfuckOperations::MovePointerRight)
                }
                BrainfuckOperations::MovePointerLeft => {
                    self.memory_pointer -= 1;
                    self.program_counter = Some(BrainfuckOperations::MovePointerLeft)
                }
                BrainfuckOperations::OutputCommand => {
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

                BrainfuckOperations::InputCommand => {
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
                _ => {}
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
impl<Display, Input> Interpreter<Display, Input>
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

        interpeter.load_ast_program(vec![]);

        let error = interpeter.run().unwrap_err();

        assert_eq!(error, InterpreterErrors::EmptyAST)
    }

    #[test]
    fn give_an_ast_that_output_a_ascii_code_when_interpreter_is_run_then_display_a_ascii_value() {
        let mut interpeter = Interpreter::new(NoRender, NoInput);

        let mut ast: Vec<BrainfuckOperations> = repeat_n(0, 65)
            .map(|_value| BrainfuckOperations::IncrementByOneCurrentCell)
            .collect();

        ast.push(BrainfuckOperations::OutputCommand);

        interpeter.load_ast_program(ast);

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

        let ast = vec![BrainfuckOperations::MovePointerRight];

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

        let ast = vec![
            BrainfuckOperations::MovePointerRight,
            BrainfuckOperations::MovePointerRight,
            BrainfuckOperations::MovePointerLeft,
        ];

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

        let ast = vec![BrainfuckOperations::InputCommand];

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
}
