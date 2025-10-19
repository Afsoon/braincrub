use core::ascii;

use crate::{
    io::{BrainfuckMemory, InputValue, MemoryErrors, MemoryTape, OutputValue, ProgramValue},
    parser::{BrainfuckNodeAST, BrainfuckOperations},
};

pub struct Interpreter<'a, Display, Input, Memory>
where
    Memory: MemoryTape<u8>,
    Display: OutputValue,
    Input: InputValue,
{
    pub memory: Memory,
    pub ast_program: Option<&'a Vec<BrainfuckNodeAST>>,
    pub program_counter: Option<BrainfuckOperations>,
    pub display: Display,
    pub input: Input,
}

#[derive(Debug, PartialEq)]
pub enum InterpreterErrors {
    EmptyAST,
    UnknownASTNode,
    OutOfRangeMemoryAccess,
}

impl<'a, Display, Input, Memory> Interpreter<'a, Display, Input, Memory>
where
    Memory: MemoryTape<u8>,
    Display: OutputValue,
    Input: InputValue,
{
    pub fn new(display: Display, input: Input, memory: Memory) -> Self {
        Interpreter {
            memory,
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
                    let _ = self.memory.update_memory_cell_value(|value| {
                        value
                            .checked_add(1)
                            .map_or_else(|| Err(MemoryErrors::CellOverflow), Ok)
                    });
                    self.program_counter = Some(BrainfuckOperations::IncrementByOneCurrentCell)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::DecrementByOneCurrentCell =>
                {
                    position = command.next_position;
                    let _ = self.memory.update_memory_cell_value(|value| {
                        value
                            .checked_sub(1)
                            .map_or_else(|| Err(MemoryErrors::CellUnderflow), Ok)
                    });
                    self.program_counter = Some(BrainfuckOperations::DecrementByOneCurrentCell)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::MovePointerRight =>
                {
                    position = command.next_position;
                    let result_move = self.memory.move_pointer_position(1);

                    if result_move.is_err() {
                        return Err(InterpreterErrors::OutOfRangeMemoryAccess);
                    }

                    self.program_counter = Some(BrainfuckOperations::MovePointerRight)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::MovePointerLeft =>
                {
                    position = command.next_position;

                    let result_move = self.memory.move_pointer_position(-1);

                    if result_move.is_err() {
                        return Err(InterpreterErrors::OutOfRangeMemoryAccess);
                    }

                    self.program_counter = Some(BrainfuckOperations::MovePointerLeft)
                }
                BrainfuckNodeAST::Command(command)
                    if command.operation == BrainfuckOperations::OutputCommand =>
                {
                    position = command.next_position;
                    match ascii::Char::from_u8(self.memory.get_current_cell_value()) {
                        Some(character) => {
                            self.display.print(ProgramValue::new(character.to_char()))
                        }
                        None => {
                            println!(
                                "Not valid ascii value, the current value is {:?}",
                                self.memory.get_current_cell_value()
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
                            let _ = self
                                .memory
                                .update_memory_cell_value(|_value| Ok(value.into()));
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
                    if self.memory.get_current_cell_value() > 0 {
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
impl<'a, Display, Input, Memory> Interpreter<'a, Display, Input, Memory>
where
    Display: OutputValue,
    Input: InputValue,
    Memory: MemoryTape<u8>,
{
    fn get_debug_info_current_position(&self) -> DebugMemoryPosition {
        DebugMemoryPosition {
            position: self.memory.get_position(),
            raw_value: self.memory.get_current_cell_value(),
            ascii_value: ascii::Char::from_u8(self.memory.get_current_cell_value())
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
        let mut interpeter = Interpreter::new(NoRender, NoInput, BrainfuckMemory::default());
        let builder = BrainfuckASTBuilder::new();

        interpeter.load_ast_program(&builder.ast);

        let error = interpeter.run().unwrap_err();

        assert_eq!(error, InterpreterErrors::EmptyAST)
    }

    #[test]
    fn give_an_ast_that_output_a_ascii_code_when_interpreter_is_run_then_display_a_ascii_value() {
        let mut interpeter = Interpreter::new(NoRender, NoInput, BrainfuckMemory::default());
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
        let mut interpeter = Interpreter::new(NoRender, NoInput, BrainfuckMemory::default());
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
        let mut interpeter = Interpreter::new(NoRender, NoInput, BrainfuckMemory::default());
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

        let mut interpeter = Interpreter::new(NoRender, AutomaticInput, BrainfuckMemory::default());

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
        let mut interpeter = Interpreter::new(NoRender, NoInput, BrainfuckMemory::default());
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
