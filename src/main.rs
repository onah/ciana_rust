/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/

extern crate ciana_rust;

use std::env;
use std::process;

use ciana_rust::ImpactLocation;

fn main() {
    let args: Vec<String> = env::args().collect();
    let target = ImpactLocation::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

   if let Err(e) = ciana_rust::run(target) {
       println!("Application error: {}", e);
       process::exit(2);
   }
}

