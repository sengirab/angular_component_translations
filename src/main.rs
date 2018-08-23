#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use component::{AngularComponent, ComponentType};
use component::TranslationResponse;
use regex::CaptureMatches;
use regex::Regex;
use regex::RegexSet;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use utilities::{create_translate_file, replace_extension, return_components};

mod component;
mod utilities;
mod route;

lazy_static! {
    static ref ROUTES: Regex = Regex::new(r"(?m)Routes\s?=\s?(\[\s[^;]*);").unwrap();
    static ref ROUTE: Regex = Regex::new(r"(?ms)(\{.*?children[^]]*\].*?}|\{.*?})").unwrap();

    static ref PATH: Regex = Regex::new(r#"(?m)path:\s['"`](.*?)['"`]"#).unwrap();
    static ref COMPONENT: Regex = Regex::new(r"(?m)component:\s?(\w+)").unwrap();
    static ref LOAD: Regex = Regex::new(r"(?mis)(?:\schildren.*#)|(\w+\.\w+)#").unwrap();
    static ref CHILDREN: Regex = Regex::new(r"(?ms)children: \[(.*?)\]").unwrap();
    static ref COMPONENTS: Regex = Regex::new(r"(?m)<(app-\w+)").unwrap();
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
                let res = TranslationResponse {
                    routes: get_routes(&routes[main_route]),
                    components: STATE.read().unwrap().to_owned(),
                };

                create_translate_file(res);
                break;
            }
        };
    }
}

fn get_routes(component: &AngularComponent) -> HashMap<String, Vec<String>> {
    let content = component.open_ts();
    let routes = capture_group(ROUTES.captures_iter(&content));

    // Initially give entry route folder routes. Determine from here on.
    // Add "path" and "hash map" as params, hash map is used to add routes.
    // path can be used to concat last path.
    let mut map = HashMap::new();
    setup_route_hierarchy(&routes.unwrap(), &String::new(), &mut map);

    map
}

fn setup_route_hierarchy(routes: &String, path: &String, map: &mut HashMap<String, Vec<String>>) {
    for route in ROUTE.captures_iter(routes) {
        let group: &str = &route[1];
        // Match all different types in here
        let set = RegexSet::new(&[PATH.as_str(), COMPONENT.as_str(), LOAD.as_str(), CHILDREN.as_str()]).unwrap();
        let set: Vec<_> = set.matches(group).into_iter().collect();

        // Clone path for each route found. We want a full path PER main route,
        let mut path = path.clone();
        for item in set {
            match item {
                // Concat path. Only thing to do here is concat, so components can be added.
                0 => {
                    let mut matches = capture_group(PATH.captures_iter(group));
                    path.push_str(&matches.unwrap());

                    if path.is_empty() {
                        path.push_str("/");
                    }
                }
                // Search for components in components.
                // Most important function here. Add components to routes in here.
                1 => {
                    let matches = capture_group(COMPONENT.captures_iter(group));
                    println!("Found component on {:?} on path: {:?}", matches, path);
                    let matches = matches.unwrap();
                    find_components(&matches);

                    let components = map.entry(path.clone()).or_insert(Vec::new());
                    components.push(matches)
                }
                // Find file that's being loaded and go recursive
                2 => {
                    let matches = capture_group(LOAD.captures_iter(group));

                    if let Some(c) = capture_group(LOAD.captures_iter(group)) {
                        let file_name = replace_extension(&c, "routing.ts");
                        let state = STATE.read().unwrap();

                        let component = state.iter().find(|c| c.file_name == file_name).unwrap();
                        let routes = capture_group(ROUTES.captures_iter(&component.open_ts()));

                        setup_route_hierarchy(&routes.unwrap(), &path, map);
                    }
                }
                // Go recursive with matches.
                3 => {
                    let matches = capture_group(CHILDREN.captures_iter(group));
                    setup_route_hierarchy(&matches.unwrap(), &path, map);
                }
                _ => {}
            }
        }
    }
}

fn find_components(name: &String) {
    // find component
    // refactor to use vec filter with only components
    let state = STATE.read().unwrap();
    let component = state.iter().find(|c| &c.name == name).unwrap();
    let component = component.open_html();

    // find components in html
    let mut components: Vec<String> = COMPONENTS.captures_iter(&component).into_iter().map(|c| c[1].to_string()).collect();
    components.dedup();

    println!("Amount of components on route {:?} \n", components);
}

fn capture_group(captures: CaptureMatches) -> Option<String> {
    captures
        .take(1)
        .fold(None, |res, item| {
            if let Some(i) = item.get(1) {
                return Some(item[1].to_string());
            }

            None
        })
}
