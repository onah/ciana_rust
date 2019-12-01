/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
extern crate clang;

use crate::ast_reader::AstReader;
use crate::SourceLocation;
use crate::CianaError;

pub struct LibClangAstReader {
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

impl AstReader for LibClangAstReader {

  fn get_reference_location(&self, target: &SourceLocation) -> Result<SourceLocation, CianaError> {
      let cl = clang::Clang::new()?;
      let index = clang::Index::new(&cl, false, false);
      let tu = index.parser(target.filename.clone()).parse()?;
      let entity = tu.get_entity();
      match visitor_children_for_reference(entity, &target) {
          Some(v) => Ok(v),
          None => Err(CianaError::NoTarget),
      }
  }
  
  fn get_same_variables_location(target: &SourceLocation) -> Result<Vec<SourceLocation>, CianaError> {
      let search_paths = if is_global_variable(target)? {
          get_all_complie_source_file()?
      } else {
          vec![target.filename.clone().to_path_buf()]
      };
  
      let mut all_results = Vec::new();
  
      let cl = clang::Clang::new()?;
      let index = clang::Index::new(&cl, false, false);
  
      for path in search_paths {
          let tu = index.parser(path).parse()?;
          let entity = tu.get_entity();
  
          let results = visitor_children_for_variables(entity, &target);
          all_results.extend(results);
      }
  
      Ok(all_results)
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
      if entity.get_kind() == clang::EntityKind::FunctionDecl {
          is_found = true;
      }
  
      if is_same_target(&entity, target) {
          return !is_found;
      }
  
      let children = entity.get_children();
      for child in children.iter() {
          if visitor_children_for_global_variable(*child, target, is_found) {
              return true;
          }
      }
      false
  }
  
  fn get_all_complie_source_file() -> Result<Vec<std::path::PathBuf>, CianaError> {
      let mut results = Vec::new();
  
      let compilation_database_path = get_compilation_database_path()?;
      let compdb = match clang::CompilationDatabase::from_directory(compilation_database_path) {
          Ok(v) => v,
          Err(_e) => {
              return Err(CianaError::Message(String::from(
                  "CompilationDatabase Open Error",
              )))
          }
      };
  
      for command in compdb.get_all_compile_commands().get_commands().iter() {
          let path = command.get_arguments()[4].clone();
          let path = absolute_to_relative(&std::path::PathBuf::from(path))?;
          results.push(path);
      }
  
      Ok(results)
  }
  
  fn visitor_children_for_variables(
      entity: clang::Entity,
      target: &SourceLocation,
  ) -> Vec<SourceLocation> {
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
            line: 2,
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
