#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use component::{AngularComponent, ComponentType};
use regex::CaptureMatches;
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
    static ref ROUTES: Regex = Regex::new(r"(?m)Routes\s?=\s?(\[\s[^;]*);").unwrap();
    static ref ROUTE: Regex = Regex::new(r"(?m)\{([^}]*)},?").unwrap();

    static ref PATH: Regex = Regex::new(r#"(?m)\spath:\s['"`](.*)['"`]"#).unwrap();
    static ref COMPONENT: Regex = Regex::new(r"(?m)[^,|^{]\scomponent:\s(.*)").unwrap();
    static ref LOAD: Regex = Regex::new(r#"(?m)\sloadChildren:\s['"`].*/(.*)#.*['"`]"#).unwrap();
    static ref CHILDREN: Regex = Regex::new(r"(?m)children:\s?\[(\s[^;]*)][^;]").unwrap();
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
    let routes = capture_group(ROUTES.captures_iter(&content));

    // Initially give entry route folder routes. Determine from here on.
    setup_route_hierarchy(&routes);
}

fn setup_route_hierarchy(routes: &String) {
    for route in ROUTE.captures_iter(routes) {
        println!("ROUTe, {:?}", route);
        let group: &str = &route[1];
        // Match all different types in here
        let set = RegexSet::new(&[PATH.as_str(), COMPONENT.as_str(), LOAD.as_str(), CHILDREN.as_str()]).unwrap();
        let set: Vec<_> = set.matches(group).into_iter().collect();

        println!("SET, {:?}", set);
        for item in set {
            match item {
                // Concat path.
                0 => {
                    let path = capture_group(PATH.captures_iter(group));
                    println!("Got path {:?}", path);
                }
                // Search for components in components.
                1 => {
                    let path = capture_group(COMPONENT.captures_iter(group));
                    println!("Got component {:?}", path);
                }
                // Find file that's being loaded and go recursive
                2 => {
                    let path = capture_group(LOAD.captures_iter(group));
                    println!("Got loaded children {:?}", path);
                }
                // Go recursive with matches.
                3 => {
                    let path = capture_group(CHILDREN.captures_iter(group));
                    println!("Got children {:?}", path);
                }
                _ => {}
            }
        }
    }
}

fn capture_group(captures: CaptureMatches) -> String {
    captures
        .fold(String::new(), |res, item| item[1].to_string())
}
