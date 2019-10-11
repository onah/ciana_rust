/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
use ciana::SourceLocation;
use std::fmt;
use std::num;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Lengths,
    Parse(num::ParseIntError),
}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> ParseError {
        ParseError::Parse(err)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Lengths => write!(f, "not enoght argmuments"),
            ParseError::Parse(ref err) => write!(f, "Parse error: {}", err),
        }
    }
}

pub fn run(args: &[String]) -> Result<SourceLocation, ParseError> {
    if args.len() < 4 {
        return Err(ParseError::Lengths);
    }

    let filename = args[1].clone();
    let line = args[2].parse()?;
    let column = args[3].parse()?;

    Ok(SourceLocation::new(filename, line, column))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_normal_case() {
        let args = vec![
            String::from("prog_name"),
            String::from("filename"),
            String::from("1"),
            String::from("2"),
        ];

        let result = run(&args);
        let correct = SourceLocation::new(String::from("filename"), 1, 2);
        assert_eq!(result.unwrap(), correct);
    }

    #[test]
    fn parse_args_not_enogth_case() {
        let args = vec![
            String::from("prog_name"),
            String::from("filename"),
            String::from("1"),
        ];

        let result = run(&args);
        assert_eq!(result.unwrap_err(), ParseError::Lengths);
    }

    #[test]
    fn parse_args_parse_error1() {
        let args = vec![
            String::from("prog_name"),
            String::from("filename"),
            String::from("foo"),
            String::from("2"),
        ];

        let result = run(&args);
        assert!(result.is_err());
    }

    #[test]
    fn parse_args_parse_error2() {
        let args = vec![
            String::from("prog_name"),
            String::from("filename"),
            String::from("1"),
            String::from("bar"),
        ];

        let result = run(&args);
        assert!(result.is_err());
    }
}
