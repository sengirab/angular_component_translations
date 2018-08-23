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
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use utilities::{create_translate_file, return_components};

mod component;
mod utilities;
mod route;

lazy_static! {
    static ref ROUTES: Regex = Regex::new(r"(?m)Routes\s?=\s?(\[\s[^;]*);").unwrap();
    static ref ROUTE: Regex = Regex::new(r"(?ms)(\{.*?children[^]]*\].*?}|\{.*?})").unwrap();

    static ref PATH: Regex = Regex::new(r#"(?m)path:\s['"`](.*?)['"`]"#).unwrap();
    static ref COMPONENT: Regex = Regex::new(r"(?m)component:\s?(.*)[}|,]").unwrap();
    static ref LOAD: Regex = Regex::new(r#"(?m)\sloadChildren:\s['"`].*/(.*)#.*['"`]"#).unwrap();
    static ref CHILDREN: Regex = Regex::new(r"(?ms)children: \[(.*?)\]").unwrap();
}

lazy_static! {
    static ref STATE: Arc<RwLock<Vec<AngularComponent>>> = Arc::new(RwLock::new(vec![]));
}

fn main() {
    let _ = &env::current_dir().unwrap();
    let path = &Path::new("../../Jasper/Dynasource.Angular/apps/portal/");

    let vec = Vec::new();
    let vec = return_components(path, vec);

    STATE.write().unwrap().extend(vec);

    // Choose main route file
    let routes: Vec<AngularComponent> = STATE.read().unwrap().iter().filter(|component| {
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

    create_translate_file(STATE.read().unwrap().to_owned());
}

fn get_routes(component: &AngularComponent) {
    let content = component.open_ts();
    let routes = capture_group(ROUTES.captures_iter(&content));

    // Initially give entry route folder routes. Determine from here on.
    // Add "path" and "hash map" as params, hash map is used to add routes.
    // path can be used to concat last path.
    let mut map = HashMap::new();
    setup_route_hierarchy(&routes, &String::new(), &mut map);
}

fn setup_route_hierarchy(routes: &String, path: &String, map: &mut HashMap<String, String>) {
    for route in ROUTE.captures_iter(routes) {
        let group: &str = &route[1];
        // Match all different types in here
        let set = RegexSet::new(&[PATH.as_str(), COMPONENT.as_str(), LOAD.as_str(), CHILDREN.as_str()]).unwrap();
        let set: Vec<_> = set.matches(group).into_iter().collect();

        // Clone path for each route found. We want a full path PER main route,
        let mut path = path.clone();
        for item in set {
            match item {
                // Concat path.
                0 => {
                    let matches = capture_group(PATH.captures_iter(group));
                    path.push_str("/");
                    path.push_str(&matches);
                    println!("Got path {:?}", path);
                }
                // Search for components in components.
                1 => {
                    let matches = capture_group(COMPONENT.captures_iter(group));
                    println!("Got component {:?}", matches);
                }
                // Find file that's being loaded and go recursive
                2 => {
                    let matches = capture_group(LOAD.captures_iter(group));
                    let vec = matches.split(".");
                    let mut vec = vec.collect::<Vec<&str>>();

                    vec.pop();
                    vec.push(&"routing.ts");
                    let file_name = vec.join(".");
                    let state = STATE.read().unwrap();
                    let component = state.iter().find(|c| c.file_name == file_name).unwrap();
                    let routes = capture_group(ROUTES.captures_iter(&component.open_ts()));
                    setup_route_hierarchy(&routes, &path, map);

//                    println!("Got loaded children {:?}, go and find routing file. {:?}", matches, routes);
                }
                // Go recursive with matches.
                3 => {
                    let matches = capture_group(CHILDREN.captures_iter(group));
//                    println!("Got children {:?}", matches);
                    setup_route_hierarchy(&matches, &path, map);
                }
                _ => {}
            }
        }
    }
}

fn capture_group(captures: CaptureMatches) -> String {
    captures
        .take(1)
        .fold(String::new(), |res, item| item[1].to_string())
}
