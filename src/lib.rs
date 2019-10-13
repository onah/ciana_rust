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
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct SourceLocation {
    filename: PathBuf,
    line: u32,
    column: u32,
}

impl SourceLocation {
    pub fn new(filename: PathBuf, line: u32, column: u32) -> SourceLocation {
        SourceLocation {
            filename,
            line,
            column,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CianaError {
    Message(String),
    Source(clang::SourceError),
    NoTarget,
}

impl From<String> for CianaError {
    fn from(err: String) -> CianaError {
        CianaError::Message(err)
    }
}

impl From<clang::SourceError> for CianaError {
    fn from(err: clang::SourceError) -> CianaError {
        CianaError::Source(err)
    }
}

impl fmt::Display for CianaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CianaError::Message(ref err) => write!(f, "Clang Error {}", err),
            CianaError::Source(ref err) => write!(f, "clang::Source Error {}", err),
            CianaError::NoTarget => write!(f, "Can't find target"),
        }
    }
}

pub fn run(target: SourceLocation) -> Result<(), CianaError> {
    let result = analyze_variables(&target)?;
    print_location(&result);

    Ok(())
}

fn get_reference_location(target: &SourceLocation) -> Result<SourceLocation, CianaError> {
    //let compilation_database_path = get_compilation_database_path().unwrap();

    let cl = clang::Clang::new()?;
    let index = clang::Index::new(&cl, false, false);
    let tu = index.parser(target.filename.clone()).parse()?;

    let entity = tu.get_entity();
    match visitor_children_for_reference(entity, &target) {
        Some(v) => Ok(v),
        None => Err(CianaError::NoTarget),
    }
}

fn analyze_variables(target: &SourceLocation) -> Result<Vec<SourceLocation>, CianaError> {
    let reference = get_reference_location(&target)?;
    let variables = get_same_variables_location(&reference)?;

    Ok(variables)
}

fn get_same_variables_location(target: &SourceLocation) -> Result<Vec<SourceLocation>, CianaError> {
    let cl = clang::Clang::new()?;
    let index = clang::Index::new(&cl, false, false);
    let tu = index.parser(target.filename.clone()).parse()?;

    let entity = tu.get_entity();

    let results = visitor_children_for_variables(entity, &target);
    Ok(results)
}

fn visitor_children_for_variables(
    entity: clang::Entity,
    target: &SourceLocation,
) -> Vec<SourceLocation> {
    let mut results: Vec<SourceLocation> = Vec::new();

    if let Some(r) = entity.get_reference() {
        if r != entity {
            if is_same_target(&r, target) {
                results.push(get_source_location_from_entity(&entity).unwrap());
                return results;
            }
        }
    }

    let children = entity.get_children();
    for child in children.iter() {
        let res = visitor_children_for_variables(*child, target);
        results.extend(res);
    }

    results
}

fn print_location(loc: &Vec<SourceLocation>) {
    for s in loc.iter() {
        println!("{:?}", s);
    }
}

#[allow(dead_code)]
fn get_compilation_database_path() -> Result<String, Box<dyn Error>> {
    let mut f = File::open("./.cianarc").expect("file not found.");
    let mut read_text = String::new();
    f.read_to_string(&mut read_text)
        .expect("something went wrong reading the file");

    Ok(read_text.trim().to_string())
}

fn is_same_target(entity: &clang::Entity, target: &SourceLocation) -> bool {
    //print_entiry_simple(&entity);

    let location = match entity.get_location() {
        Some(v) => v,
        None => return false,
    };

    let filelocation = location.get_file_location();
    let file = match filelocation.file {
        Some(v) => v,
        None => return false,
    };

    if file.get_path() == target.filename
        && filelocation.line == target.line
        && filelocation.column == target.column
    {
        return true;
    };

    false
}

fn get_source_location_from_entity(entity: &clang::Entity) -> Option<SourceLocation> {
    let location = match entity.get_location() {
        Some(v) => v,
        None => return None,
    };

    let filelocation = location.get_file_location();
    let file = match filelocation.file {
        Some(v) => v,
        None => return None,
    };

    Some(SourceLocation {
        filename: file.get_path(),
        line: filelocation.line,
        column: filelocation.column,
    })
}

fn get_reference_location_from_entiry(entity: &clang::Entity) -> Option<SourceLocation> {
    if let Some(v) = entity.get_reference() {
        get_source_location_from_entity(&v)
    } else {
        None
    }
}

fn visitor_children_for_reference(
    entity: clang::Entity,
    target: &SourceLocation,
) -> Option<SourceLocation> {
    let mut result: Option<SourceLocation> = None;

    if is_same_target(&entity, target) {
        result = get_reference_location_from_entiry(&entity);
    }

    let children = entity.get_children();
    for child in children.iter() {
        if let Some(v) = visitor_children_for_reference(*child, target) {
            return Some(v);
        };
    }

    return result;
}

#[allow(dead_code)]
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
    #[ignore]
    fn test_get_reference_from_local_variable() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 6,
            column: 13,
        };
        let result = get_reference_location(&target).unwrap();
        let expect = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 4,
            column: 14,
        };

        assert_eq!(result, expect);
    }

    #[test]
    #[ignore]
    fn no_target_from_get_reference() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 5,
            column: 13,
        };
        let result = get_reference_location(&target);
        assert_eq!(result.unwrap_err(), CianaError::NoTarget);
    }

    #[test]
    #[ignore]
    fn test_get_same_variable_location() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 4,
            column: 14,
        };
        let result = get_same_variables_location(&target).unwrap();
        let mut expect: Vec<SourceLocation> = Vec::new();
        expect.push(SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 6,
            column: 13,
        });
        expect.push(SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 6,
            column: 28,
        });

        assert_eq!(result, expect);
    }
}
