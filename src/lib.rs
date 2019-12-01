/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
mod ast_reader;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::ast_reader::AstReader;
use crate::ast_reader::libclang_ast_reader::LibClangAstReader;

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
    let reader = LibClangAstReader{};
    let result = analyze_variables(reader, &target)?;
    print_location(&result);

    Ok(())
}

fn analyze_variables<T: AstReader>(reader: T, target: &SourceLocation) -> Result<Vec<SourceLocation>, CianaError> {
    let reference = reader.get_reference_location(&target)?;
    let variables = reader.get_same_variables_location(&reference)?;

    Ok(variables)
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

fn print_location(loc: &[SourceLocation]) {
    for s in loc.iter() {
        println!("{:?}", s);
    }
}

fn get_compilation_database_path() -> Result<String, CianaError> {
    let mut f = File::open("./.cianarc").expect("file not found.");
    let mut read_text = String::new();
    f.read_to_string(&mut read_text)
        .expect("something went wrong reading the file");

    Ok(read_text.trim().to_string())
}
