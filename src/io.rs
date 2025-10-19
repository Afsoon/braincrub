use core::ascii;

use inquire::{CustomType, ui::RenderConfig};
use std::{fmt::Display, num::IntErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramValue(pub char);

impl ProgramValue {
    pub fn new(value: char) -> Self {
        return ProgramValue(value);
    }
}

impl Display for ProgramValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub enum InputError {
    Unknown,
}

pub trait InputValue {
    fn get_input(&self) -> Result<ProgramValue, InputError>;
}

pub struct BasicInput<'a> {
    prompt: CustomType<'a, ProgramValue>,
}

#[derive(Debug, PartialEq)]
pub enum AsciiParseError {
    NotValidNumericRangeValue,
    NotValidAsciiCharacter,
    UnknownError,
}

impl TryFrom<&str> for ProgramValue {
    type Error = AsciiParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value_parsed = value
            .to_owned()
            .parse::<u8>()
            .map(|ascii_code| ascii::Char::from_u8(ascii_code));

        match value_parsed {
            Ok(ascii_char) => ascii_char.map_or_else(
                || Err(AsciiParseError::NotValidNumericRangeValue),
                |value| Ok(ProgramValue(value.to_char())),
            ),
            Err(error) if *error.kind() == IntErrorKind::PosOverflow => {
                Err(AsciiParseError::NotValidNumericRangeValue)
            }
            Err(error) if *error.kind() == IntErrorKind::InvalidDigit => {
                if value.is_ascii() {
                    let byte_ascii_array = value.as_ascii().unwrap().to_vec();
                    return Ok(ProgramValue(byte_ascii_array[0].to_char()));
                }

                return Err(AsciiParseError::NotValidAsciiCharacter);
            }
            Err(_) => Err(AsciiParseError::UnknownError),
        }
    }
}

impl Into<u8> for ProgramValue {
    fn into(self) -> u8 {
        self.0.as_ascii().unwrap().to_u8()
    }
}

impl<'a> Default for BasicInput<'a> {
    fn default() -> Self {
        let ascii_prompt: CustomType<'a, ProgramValue> = CustomType {
            message: "Write an ascii character or his code value",
            starting_input: None,
            formatter: &|value| value.to_string(),
            default_value_formatter: &|value| {
                value.to_string()
            },
            default: None,
            validators: vec![],
            placeholder: Some("A or 65"),
            error_message: "Please type a valid ascii character".into(),
            help_message: "A valid ascii code value is in the range of 0 to 127, or if you want to type a character, those must be uppercase".into(),
            parser: &|value| ProgramValue::try_from(value).map_err(|_err| ()),
            render_config: RenderConfig::default(),
        };

        Self {
            prompt: ascii_prompt,
        }
    }
}

impl<'a> InputValue for BasicInput<'a> {
    fn get_input(&self) -> Result<ProgramValue, InputError> {
        self.prompt
            .to_owned()
            .prompt()
            .or_else(|_value| Err(InputError::Unknown))
    }
}

pub trait OutputValue {
    fn print(&self, value: ProgramValue);
}

#[derive(Copy, Clone)]
pub struct BasicOutput;

impl OutputValue for BasicOutput {
    fn print(&self, value: ProgramValue) {
        print!("{:?}", value.0)
    }
}

#[cfg(test)]
mod conversion_test {
    use crate::io::{AsciiParseError, ProgramValue};

    #[test]
    fn when_string_represent_a_valid_ascii_char_then_return_the_value_parser() {
        let ascii_char = ProgramValue::try_from("A").unwrap();

        assert_eq!(ascii_char, ProgramValue('A'))
    }

    #[test]
    fn when_string_represent_a_valid_ascii_char_code_then_return_the_value_as_char() {
        let ascii_char = ProgramValue::try_from("66").unwrap();

        assert_eq!(ascii_char, ProgramValue('B'))
    }

    #[test]
    fn when_string_have_a_value_non_ascii_complatible_then_return_an_error() {
        let ascii_char = ProgramValue::try_from("Ñ").unwrap_err();

        assert_eq!(ascii_char, AsciiParseError::NotValidAsciiCharacter)
    }

    #[test]
    fn when_string_have_a_numeric_value_not_in_range_of_u8_then_return_an_error() {
        let ascii_char = ProgramValue::try_from("256").unwrap_err();

        assert_eq!(ascii_char, AsciiParseError::NotValidNumericRangeValue)
    }

    #[test]
    fn when_string_have_a_numeric_value_greather_than_127_then_return_an_error() {
        let ascii_char = ProgramValue::try_from("128").unwrap_err();

        assert_eq!(ascii_char, AsciiParseError::NotValidNumericRangeValue)
    }
}
