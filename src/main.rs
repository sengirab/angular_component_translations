#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate pcre;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structure::AngularStructure;
use structure::component::AngularComponent;
use structure::component::ComponentType;

mod structure;

fn main() {
    let input_path = std::env::args_os().skip(1).next()
        .expect("usage: component-translations <path> <output-path> <routing-file>");
    let input_path = create_path(input_path.into_string().unwrap());

    let output_path = std::env::args_os().skip(2).next()
        .expect("usage: component-translations <path> <output-path> <routing-file>");
    let output_path = create_path(output_path.into_string().unwrap());

    let mut structure = AngularStructure::new(input_path);
    let routes: Vec<AngularComponent> = structure.get_component_by_kind(ComponentType::Route);

    let file = match std::env::args_os().skip(3).next() {
        Some(file) => file.into_string().unwrap(),
        None => {
            println!("usage: component-translations <path> <output-path> <routing-file>");
            std::process::exit(0);
        }
    };

    let component = match structure.components.get(&file) {
        Some(c) => Some(c.clone()),
        None => None
    };

    match component {
        Some(component) => {
            structure.setup_routes(&component);
            create_translate_file(structure, output_path);
        }
        None => {
            println!("Please choose one of the following routes");
            for (_i, route) in routes.iter().enumerate() {
                println!(" -> {}", route.file_name);
            }

            std::process::exit(0);
        }
    }
}

fn create_translate_file<P: AsRef<Path>>(structure: AngularStructure, path: P) {
    let path = path.as_ref().join("component_translation_keys.json");

    let json = serde_json::to_string(&structure).unwrap();
    let mut file = File::create(path)
        .expect("Failed creating file");

    file.write(json.into_bytes().as_slice())
        .expect("Failed writing file");
}

fn create_path<P: AsRef<Path>>(p: P) -> PathBuf {
    let path = env::current_exe().unwrap();
    let path = path.parent().unwrap();

    path.join(p)
}