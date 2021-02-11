//! A module to hold `Error`, etc., types generated by `error-chain`.

use csv;
use std::io;

error_chain! {
    foreign_links {
        Csv(csv::Error);
        Io(io::Error);
    }

    errors {
        NoSuchColumn(name: String) {
            description("Cannot find specified CSV column")
            display("No CSV column with name '{}'", name)
        }
        NonUtf8Zip(pos: Option<csv::Position>) {
            description("Zip code column contained non-UTF8 data")
            display("Non-UTF8 zip code data at line {:?}",
                    pos.as_ref().map(|p| p.line()))
        }
    }
}

impl Error {
    /// Return an `Error` for `ErrorKind::NoSuchColumn`.
    pub fn no_such_column<S: Into<String>>(name: S) -> Error {
        ErrorKind::NoSuchColumn(name.into()).into()
    }

    pub fn non_utf8_zip(pos: Option<&csv::Position>) -> Error {
        ErrorKind::NonUtf8Zip(pos.map(|p| p.to_owned())).into()
    }
}
