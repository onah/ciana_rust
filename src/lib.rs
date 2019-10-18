/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
extern crate clang;

use std::env;
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
    IoError,
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

impl From<std::io::Error> for CianaError {
    fn from(_err: std::io::Error) -> CianaError {
        CianaError::IoError
    }
}

impl fmt::Display for CianaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CianaError::Message(ref err) => write!(f, "Clang Error {}", err),
            CianaError::Source(ref err) => write!(f, "clang::Source Error {}", err),
            CianaError::NoTarget => write!(f, "Can't find target"),
            CianaError::IoError => write!(f, "Io Error"),
        }
    }
}

pub fn run(target: SourceLocation) -> Result<(), CianaError> {
    let result = analyze_variables(&target)?;
    print_location(&result);

    Ok(())
}

fn get_reference_location(target: &SourceLocation) -> Result<SourceLocation, CianaError> {
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

fn is_global_variable(target: &SourceLocation) -> Result<bool, CianaError> {
    let cl = clang::Clang::new()?;
    let index = clang::Index::new(&cl, false, false);
    let tu = index.parser(target.filename.clone()).parse()?;

    let entity = tu.get_entity();

    let result = visitor_children_for_global_variable(entity, &target, false);
    Ok(result)
}

fn visitor_children_for_global_variable(
    entity: clang::Entity,
    target: &SourceLocation,
    mut is_found: bool,
) -> bool {
    if entity.get_kind() == clang::EntityKind::VarDecl && is_found {
        is_found = true;
    }

    if is_same_target(&entity, target) {
        if is_found {
            return false;
        } else {
            return true;
        }
    }

    let children = entity.get_children();
    for child in children.iter() {
        return visitor_children_for_global_variable(*child, target, is_found);
    }

    return false;
}

fn absolute_to_relative(
    absolute: &std::path::PathBuf,
) -> Result<std::path::PathBuf, std::io::Error> {
    let pwd = env::current_dir()?;
    match absolute.strip_prefix(pwd) {
        Ok(v) => Ok(v.to_path_buf()),
        Err(_e) => Ok(absolute.to_path_buf()),
    }
}

fn get_same_variables_location(target: &SourceLocation) -> Result<Vec<SourceLocation>, CianaError> {
    let global_variable = is_global_variable(target)?;

    let cl = clang::Clang::new()?;
    let index = clang::Index::new(&cl, false, false);

    if global_variable {
        let mut all_results = Vec::new();
        let compilation_database_path = get_compilation_database_path().unwrap();
        let compdb = clang::CompilationDatabase::from_directory(compilation_database_path).unwrap();
        for command in compdb.get_all_compile_commands().get_commands().iter() {
            let path = command.get_arguments()[4].clone();
            let path = absolute_to_relative(&std::path::PathBuf::from(path))?;
            //println!("{:?}", new_path);

            let tu = index.parser(path).parse()?;
            let entity = tu.get_entity();

            let results = visitor_children_for_variables(entity, &target);

            all_results.extend(results);
        }
        return Ok(all_results);
    }

    let tu = index.parser(target.filename.clone()).parse()?;
    let entity = tu.get_entity();

    let results = visitor_children_for_variables(entity, &target);
    Ok(results)
}

fn visitor_children_for_variables(
    entity: clang::Entity,
    target: &SourceLocation,
) -> Vec<SourceLocation> {
    // print_entiry_simple(&entity);

    let mut results: Vec<SourceLocation> = Vec::new();

    // TODO: whether to exclude if the reference is yourself
    if let Some(r) = entity.get_reference() {
        if r != entity && is_same_target(&r, target) {
            results.push(get_source_location_from_entity(&entity).unwrap());
            return results;
        }
    }

    let children = entity.get_children();
    for child in children.iter() {
        let res = visitor_children_for_variables(*child, target);
        results.extend(res);
    }

    results
}

fn print_location(loc: &[SourceLocation]) {
    for s in loc.iter() {
        println!("{:?}", s);
    }
}

fn is_same_target(entity: &clang::Entity, target: &SourceLocation) -> bool {
    //print_entiry_simple(&entity);
    let comparison = match get_source_location_from_entity(entity) {
        Some(v) => v,
        None => return false,
    };

    &comparison == target
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
    let result: Option<SourceLocation> = if is_same_target(&entity, target) {
        get_reference_location_from_entiry(&entity)
    } else {
        None
    };

    let children = entity.get_children();
    for child in children.iter() {
        if let Some(v) = visitor_children_for_reference(*child, target) {
            return Some(v);
        };
    }
    result
}

fn get_compilation_database_path() -> Result<String, CianaError> {
    let mut f = File::open("./.cianarc").expect("file not found.");
    let mut read_text = String::new();
    f.read_to_string(&mut read_text)
        .expect("something went wrong reading the file");

    Ok(read_text.trim().to_string())
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
        let result = get_reference_location(&target);
        let expect = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 4,
            column: 14,
        };

        assert_eq!(result, Ok(expect));
    }

    #[test]
    #[ignore]
    fn test_analyze_variables_from_local_variable() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/main.c"),
            line: 7,
            column: 8,
        };
        let result = analyze_variables(&target);
        let mut expect: Vec<SourceLocation> = Vec::new();
        expect.push(SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/main.c"),
            line: 8,
            column: 12,
        });
        expect.push(SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/main.c"),
            line: 10,
            column: 19,
        });

        assert_eq!(result, Ok(expect));
    }

    #[test]
    #[ignore]
    fn test_analyze_variables_from_global_variable() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 8,
            column: 3,
        };
        let result = analyze_variables(&target);
        let mut expect: Vec<SourceLocation> = Vec::new();
        expect.push(SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 8,
            column: 3,
        });
        expect.push(SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/subfunc.c"),
            line: 10,
            column: 3,
        });
        assert_eq!(result, Ok(expect));
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
        assert_eq!(result, Err(CianaError::NoTarget));
    }

    #[test]
    #[ignore]
    fn test_get_same_variable_location() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 4,
            column: 14,
        };
        let result = get_same_variables_location(&target);
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

        assert_eq!(result, Ok(expect));
    }

    #[test]
    #[ignore]
    fn test_is_global_variable_for_local() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/func.c"),
            line: 4,
            column: 14,
        };
        let result = is_global_variable(&target);
        assert_eq!(result, Ok(false));
    }

    #[test]
    #[ignore]
    fn test_is_global_variable_from_global() {
        let target = SourceLocation {
            filename: PathBuf::from("test_project/c_variable/src/subfunc.h"),
            line: 1,
            column: 5,
        };
        let result = is_global_variable(&target);
        assert_eq!(result, Ok(true));
    }

    #[test]
    #[ignore]
    fn test_absolute_to_relative() {
        let pwd = env::current_dir().expect("test_absolute_to_relative current_dir error");
        let input = pwd.join("tmp/test.txt");
        let result = absolute_to_relative(&input);
        assert_eq!(
            result
                .expect("test_absolute_to_relative result error")
                .to_str(),
            Some("tmp/test.txt")
        );
    }
}
