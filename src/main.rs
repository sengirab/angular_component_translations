#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate pcre;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use component::{AngularComponent, ComponentType};
use component::AngularComponents;
use component::TranslationResponse;
use pcre::Pcre;
use regex::CaptureMatches;
use regex::Regex;
use regex::RegexSet;
use route::AngularRoutes;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use utilities::{create_translate_file, replace_extension, return_components};
use utilities::capture_group;
use utilities::selector_to_component_name;

mod component;
mod utilities;
mod route;

lazy_static! {
    static ref ROUTES: Regex = Regex::new(r"(?m)Routes\s?=\s?(\[\s[^;]*);").unwrap();

    static ref PATH: Regex = Regex::new(r#"(?m)path:\s['"`](.*?)['"`]"#).unwrap();
    static ref COMPONENT: Regex = Regex::new(r"(?m)component:\s?(\w+)").unwrap();
    static ref LOAD: Regex = Regex::new(r"(?mis)(?:\schildren.*#)|(\w+\.\w+)#").unwrap();
    static ref CHILDREN: Regex = Regex::new(r"(?ms)children: \[(.*?)^[[:blank:]]{8}]").unwrap();
    static ref COMPONENTS: Regex = Regex::new(r"(?m)<(app-(?:\w+-?)*)").unwrap();
    static ref ENTRY: Regex = Regex::new(r"(?sm)entryComponents: \[\s*(.*)]").unwrap();

}

lazy_static! {
    static ref STATE: Arc<RwLock<AngularComponents> = Arc::new(RwLock::new(HashMap::new()));
}

struct AngularStructure {
    routes: AngularRoutes,
    components: AngularComponents,
}

impl AngularStructure {
    pub fn new(path: &Path) -> AngularStructure {
        let structure = AngularStructure {
            components: Self::setup_components(path),
            routes: AngularRoutes { value: HashMap::new() },
        };

        structure
    }

    pub fn setup_routes(&mut self, component: &AngularComponent) {}

    pub fn get_component_by_kind(&self, kind: ComponentType) -> Vec<AngularComponent> {
        self.components.iter().map(|(_, component)| component).filter(|component| {
            if component_kind == component.kind {
                return true;
            }

            false
        }).cloned().collect()
    }

    fn setup_components(path: &Path) -> AngularComponents {
        AngularComponents::new(path)
    }

    fn find_components(&self, name: &str, found: &mut Vec<String>) {
        let component = self.components.get(name).unwrap();
        let components = component.get_used_components();

        for component in &components {
            found.push(component.clone());
            self.find_components(component, found);
        }
    }
}

fn main() {
    let _ = &env::current_dir().unwrap();
    let path = &Path::new("../../Jasper/Dynasource.Angular/apps/portal/");

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
                create_translate_file(res);

                break;
            }
        };
    }
}