use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use thiserror::Error;

/**
 * io::Error doesn't implement PartialEq but I can't implement PartialEq as the type
 * don't belong to my crate. I'm wrapping the io::Error on my own struct and implement
 * the PartialEq on my own. This case checking that both error kind are equals. Then I'm
 * using the suggestion of thiserror to hide the error implementation from the public API,
 * then I can propagate the error without problem.
 */
#[derive(Error, Debug)]
#[error(transparent)]
pub struct PublicError(#[from] io::Error);

impl PartialEq for PublicError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum FileError {
    #[error("The file {file_name:?} located in {path:?} doesn't exist")]
    FileNotFound { file_name: String, path: String },
    #[error("The ${path:?} doesn't point to a file, review it")]
    FilePathMalformed { path: String },
    #[error("The ${path:?} doesn't point to a file, it's a directory")]
    IsADirectory { path: String },
    #[error("Unable to read the file due lack of permission")]
    NotEnoughPermission,
    #[error("Unexpected error processing the file")]
    UnexpectedError(#[from] PublicError),
}

fn get_file_name_string(path: &str) -> Option<String> {
    let path_normalized = Path::new(path);

    match path_normalized.file_name() {
        Some(file_name) => Some(file_name.to_string_lossy().to_string()),
        None => None,
    }
}

fn get_ancestor_path(path: &str) -> String {
    let path_normalized = Path::new(path);

    let mut parent_paths = path_normalized.ancestors();
    parent_paths.next();

    parent_paths
        .next()
        .map(|parent_path| parent_path.to_string_lossy().to_string())
        .unwrap_or_else(|| String::new())
}

pub fn read_source_code_file(path: &str) -> Result<String, FileError> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == ErrorKind::NotFound => match get_file_name_string(path) {
            Some(file_name) => Err(FileError::FileNotFound {
                file_name,
                path: get_ancestor_path(path),
            }),
            None => Err(FileError::UnexpectedError(PublicError(error))),
        },
        Err(error) if error.kind() == ErrorKind::NotADirectory => {
            Err(FileError::FilePathMalformed {
                path: path.to_string(),
            })
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            Err(FileError::NotEnoughPermission)
        }
        Err(error) if error.kind() == ErrorKind::IsADirectory => Err(FileError::IsADirectory {
            path: path.to_string(),
        }),
        Err(error) => Err(FileError::UnexpectedError(PublicError(error))),
    }
}

#[cfg(test)]
mod read_file {

    use super::*;

    // Only works in Linux and Mac
    // CARGO_MAFIDEST_DIR is a stable env to located files for unit test resources that need to read files.
    macro_rules! file_test_case {
        ($fname:expr) => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $fname)
        };
    }

    #[test]
    fn when_file_exists_then_return_the_file_content() {
        let path = file_test_case!("file_exists.txt");

        let content_file = read_source_code_file(&path).unwrap();

        assert_eq!(content_file, "+\n")
    }

    #[test]
    fn when_path_point_a_file_that_dont_exist_then_return_file_not_found_error() {
        let parent_path = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test");
        let path = file_test_case!("not_exists.txt");

        let file_error = read_source_code_file(&path).unwrap_err();

        assert_eq!(
            file_error,
            FileError::FileNotFound {
                file_name: "not_exists.txt".to_string(),
                path: parent_path.to_string(),
            }
        )
    }

    #[test]
    fn when_path_point_a_dir_then_return_is_a_directory_error() {
        let parent_path = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/");
        let path = file_test_case!("");

        let file_error = read_source_code_file(&path).unwrap_err();

        assert_eq!(
            file_error,
            FileError::IsADirectory {
                path: parent_path.to_string(),
            }
        )
    }

    #[test]
    fn when_path_is_malformed_then_return_file_path_malformed_error() {
        let parent_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/test/",
            "file_exists.txt/.."
        );
        let path = file_test_case!("file_exists.txt/..");

        let file_error = read_source_code_file(&path).unwrap_err();

        assert_eq!(
            file_error,
            FileError::FilePathMalformed {
                path: parent_path.to_string(),
            }
        )
    }

    #[test]
    #[ignore = "I can't commit a file wihtout read permission, it should be update to create a tmp file without permission and read it"]
    fn given_user_with_lack_of_permission_when_user_try_to_read_a_file_without_permission_then_return_not_enough_permission_error()
     {
        let path = file_test_case!("not_permission.txt");

        let file_error = read_source_code_file(&path).unwrap_err();

        assert_eq!(file_error, FileError::NotEnoughPermission)
    }
}
