/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
extern crate clang;

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
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

pub fn parse_args(args: &[String]) -> Result<SourceLocation, ParseError> {
    if args.len() < 4 {
        return Err(ParseError::Lengths);
    }

    let filename = args[1].clone();
    let line = args[2].parse()?;
    let column = args[3].parse()?;

    Ok(SourceLocation {
        filename,
        line,
        column,
    })
}

#[derive(Debug, PartialEq)]
pub struct SourceLocation {
    filename: String,
    line: u32,
    column: u32,
}

fn get_reference_location(target: &SourceLocation) -> SourceLocation {
    SourceLocation {
        filename: "test_project/c_variable/src/func.c".to_string(),
        line: 1,
        column: 1,
    }
}

pub fn run(target: SourceLocation) -> Result<(), Box<dyn Error>> {
    //let compilation_database_path = get_compilation_database_path().unwrap();

    let cl = clang::Clang::new().unwrap();
    let index = clang::Index::new(&cl, false, false);

    let astfile = "";

    let file = target.filename.clone();
    let tu = clang::TranslationUnit::from_ast(&index, astfile)
        .map_err(|_e| "TranslationUnit Error".to_owned())?;

    let entity = tu.get_entity();
    visitor_children(entity, &target);

    Ok(())
}

fn get_compilation_database_path() -> Result<String, Box<dyn Error>> {
    let mut f = File::open("./.cianarc").expect("file not found.");
    let mut read_text = String::new();
    f.read_to_string(&mut read_text)
        .expect("something went wrong reading the file");

    Ok(read_text.trim().to_string())
}

fn check_target(entity: &clang::Entity, target: &SourceLocation) {
    let location = match entity.get_location() {
        None => return,
        Some(value) => value.get_file_location(),
    };

    if location.file.unwrap().get_path().to_str().unwrap() == target.filename
        && location.line == target.line
        && location.column == target.column
    {
        println!("ent");
        print_entiry_simple(&entity);

        println!("  def");
        print!("  ");
        match entity.get_definition() {
            None => println!(""),
            Some(v) => print_entiry_simple(&v),
        };

        println!("  ref");
        print!("  ");
        match entity.get_reference() {
            None => println!(""),
            Some(v) => print_entiry_simple(&v),
        };
    }
}

fn visitor_children(entity: clang::Entity, target: &SourceLocation) {
    check_target(&entity, target);

    let children = entity.get_children();
    for child in children.iter() {
        visitor_children(*child, target);
    }
}

fn print_entiry_simple(entity: &clang::Entity) {
    let location = match entity.get_location() {
        None => None,
        Some(value) => Some(value.get_file_location()),
    };

    match location {
        None => print!(""),
        Some(loc) => print!(
            "{}|{} col {}:",
            loc.file.unwrap().get_path().to_str().unwrap(),
            loc.line,
            loc.column
        ),
    };

    let kind = entity.get_kind();
    print!("{:?}:", kind);

    let name = entity.get_display_name();
    let name = match name {
        Some(n) => n,
        None => std::string::String::new(),
    };

    println!("{}", name);
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

        let result = parse_args(&args);
        let correct = SourceLocation {
            filename: String::from("filename"),
            line: 1,
            column: 2,
        };
        assert_eq!(result.unwrap(), correct);
    }

    #[test]
    fn parse_args_not_enogth_case() {
        let args = vec![
            String::from("prog_name"),
            String::from("filename"),
            String::from("1"),
        ];

        let result = parse_args(&args);
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

        let result = parse_args(&args);
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

        let result = parse_args(&args);
        assert!(result.is_err()); 
    }


    #[test]
    fn test_get_reference_from_local_variable() {
        let target = SourceLocation {
            filename: "test_project/c_variable/src/func.c".to_string(),
            line: 5,
            column: 13,
        };
        let result = get_reference_location(&target);
        let expect = SourceLocation {
            filename: "test_project/c_variable/src/func.c".to_string(),
            line: 1,
            column: 1,
        };

        assert_eq!(result, expect);
    }
}
