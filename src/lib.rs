/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/

extern crate clang;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

pub struct ImpactLocation {
    filename: String,
    line: u32,
    column: u32,
}

impl ImpactLocation {
    pub fn new(args: &[String]) -> Result<ImpactLocation, &'static str> {
        if args.len() < 4 {
            return Err("not enough arguments");
        }

        let filename = args[1].clone();
        let line = args[2].parse().unwrap();
        let column = args[3].parse().unwrap();

        Ok(ImpactLocation { filename, line, column })
    }
}

pub fn run(target: ImpactLocation) -> Result<(), Box<dyn Error>> {

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
    let mut f =  File::open("./.cianarc").expect("file not found.");
    let mut read_text = String::new();
    f.read_to_string(&mut read_text)
        .expect("something went wrong reading the file");

    Ok(read_text.trim().to_string())
}

fn check_target(entity: &clang::Entity , target: &ImpactLocation) {
    let location = match entity.get_location() {
        None => return,
        Some(value) => value.get_file_location(),
    };

    if location.file.unwrap().get_path().to_str().unwrap() == target.filename &&
       location.line == target.line &&
       location.column == target.column {
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

fn visitor_children(entity: clang::Entity, target: &ImpactLocation) {
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
        Some(loc) => print!("{}|{} col {}:",
             loc.file.unwrap().get_path().to_str().unwrap(), loc.line, loc.column),
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

