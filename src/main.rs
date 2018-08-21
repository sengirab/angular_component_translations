#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use component::{AngularComponent, ComponentType};
use regex::Regex;
use regex::RegexSet;
use std::cmp::Ordering;
use std::env;
use std::io;
use std::path::Path;
use utilities::{create_translate_file, return_components};

mod component;
mod utilities;
mod route;

lazy_static! {
    static ref ROUTES: Regex = Regex::new(r"(?m)Routes\s?=\s?\[\s[^;]*;").unwrap();
}
lazy_static! {
    static ref ROUTE: Regex = Regex::new(r"(?m)\{([^}]*)},?").unwrap();
}

fn main() {
    let _ = &env::current_dir().unwrap();
    let path = &Path::new("../../Jasper/Dynasource.Angular/apps/portal/");

    let vec = Vec::new();
    let vec = return_components(path, vec);

    // Choose main route file
    let routes: Vec<AngularComponent> = vec.iter().filter(|component| {
        match component.kind {
            ComponentType::Route => true,
            _ => false
        }
    }).cloned().collect();

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
                get_routes(&routes[main_route]);
                break;
            }
        };
    }

    create_translate_file(vec);
}

fn get_routes(component: &AngularComponent) {
    let content = component.open_ts();
    let routes = ROUTES.captures_iter(&content)
        .fold(String::new(), |res, item| item[0].to_string());

    // Initially give entry route folder routes. Determine from here on.
    setup_route_hierarchy(&routes)
}

fn setup_route_hierarchy(routes: &String) {
    for route in ROUTE.captures_iter(routes) {
        // Match all different types in here
        let set = RegexSet::new(&[
            r#"(?m)\spath:\s['"`](.*)['"`]"#,
            r"(?m)[^,|^{]\scomponent:\s(.*)",
            r#"(?m)\sloadChildren:\s['"`].*/(.*)#.*['"`]"#,
            r"(?m)children:\s?\[\s[^;]*]" // If children are found, go recursive.
        ]).unwrap();

        println!("Going to get routes {:?}", route[1].to_string());
    }
}
