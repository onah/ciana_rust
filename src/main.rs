/**
CIANA - C/C++ Change Impact ANAlyzer

Copyright (c) 2019 HANO Hiroyuki

This software is released under MIT License,
http://opensource.org/licenses/mit-license.php
*/
extern crate ciana;

use std::env;
use std::process;

use ciana::parse_args;

fn main() {
    let args: Vec<String> = env::args().collect();
    let target = parse_args(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = ciana::run(target) {
        println!("Application error: {}", e);
        process::exit(2);
    }
}
