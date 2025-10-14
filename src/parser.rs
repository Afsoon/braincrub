#[derive(Debug, PartialEq)]
enum BrainfuckOperations {
    MovePointerRight,
    MovePointerLeft,
    IncrementByOneCurrentCell,
    DecrementByOneCurrentCell,
    InputCommand,
    OutputCommand,
    LoopStart,
    LoopEnd,
}

fn map_char_to_brainfuck_opeartion(token: char) -> Option<BrainfuckOperations> {
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

fn from_source_to_ast(source_code: &str) -> Vec<BrainfuckOperations> {
    source_code
        .chars()
        .filter_map(map_char_to_brainfuck_opeartion)
        .collect()
}

#[cfg(test)]
mod parser_source_code_test {
    use super::*;

    #[test]
    fn given_a_source_code_with_only_valid_characteres_when_parse_it_to_ast_then_return_an_ast_with_the_same_size()
     {
        let input = "[+-.<,>]";

        let result = from_source_to_ast(input);

        assert_eq!(
            result,
            [
                BrainfuckOperations::LoopStart,
                BrainfuckOperations::IncrementByOneCurrentCell,
                BrainfuckOperations::DecrementByOneCurrentCell,
                BrainfuckOperations::OutputCommand,
                BrainfuckOperations::MovePointerLeft,
                BrainfuckOperations::InputCommand,
                BrainfuckOperations::MovePointerRight,
                BrainfuckOperations::LoopEnd,
            ],
        )
    }

    #[test]
    fn given_a_source_code_with_some_invalid_characters_when_parse_it_to_ast_then_return_an_ast_with_invalid_characters_filtered()
     {
        let input = "b[de+a-]";

        let result = from_source_to_ast(input);

        assert_eq!(
            result,
            [
                BrainfuckOperations::LoopStart,
                BrainfuckOperations::IncrementByOneCurrentCell,
                BrainfuckOperations::DecrementByOneCurrentCell,
                BrainfuckOperations::LoopEnd,
            ],
        )
    }

    #[test]
    fn given_a_source_code_with_only_invalid_characters_when_parse_it_to_ast_then_return_an_empty_array()
     {
        let input = "not a none valid character!!!";

        let result = from_source_to_ast(input);

        assert_eq!(result, [],)
    }
}
