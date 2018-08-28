#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate pcre;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::cmp::Ordering;
use std::env;
use std::io;
use structure::AngularStructure;
use structure::component::AngularComponent;
use structure::component::ComponentType;
use std::fs::File;
use std::io::Write;

mod structure;

fn main() {
    let path = env::current_exe().unwrap();
    let path = path.parent().unwrap();

    let mut structure = AngularStructure::new(path);
    let routes: Vec<AngularComponent> = structure.get_component_by_kind(ComponentType::Route);

    println!("Choose main route file:");
    loop {
        for (i, route) in routes.iter().enumerate() {
            println!("{}) {}", i, route.file_name);
        }

        let mut main_route = String::new();
        io::stdin().read_line(&mut main_route)
            .expect("Failed to read line");

        let main_route: usize = match main_route.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let length = &routes.len() - 1;
        match main_route.cmp(&length) {
            Ordering::Greater => println!("Choose from the list please"),
            _ => {
                structure.setup_routes(&routes[main_route]);
                create_translate_file(structure);

                break;
            }
        };
    }
}

pub fn create_translate_file(structure: AngularStructure) {
    let path = env::current_exe().unwrap();
    let path = path.parent().unwrap();
    let path = path.join("component_translation_keys.json");


    let json = serde_json::to_string(&structure).unwrap();
    let mut file = File::create(path)
        .expect("Failed creating file");

    file.write(json.into_bytes().as_slice())
        .expect("Failed writing file");
}