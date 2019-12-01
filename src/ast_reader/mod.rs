/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
pub mod libclang_ast_reader;

use crate::SourceLocation;
use crate::CianaError;

pub trait AstReader {
    fn get_reference_location(&self, target: &SourceLocation) -> Result<SourceLocation, CianaError>;
    fn get_same_variables_location(&self, target: &SourceLocation) -> Result<Vec<SourceLocation>, CianaError>;
}

