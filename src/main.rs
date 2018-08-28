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
use route::AngularRoutes;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use utilities::{create_translate_file};

mod component;
mod utilities;
mod route;

#[derive(Debug, Serialize)]
pub struct AngularStructure {
    pub routes: AngularRoutes,
    pub components: AngularComponents,
}

impl AngularStructure {
    pub fn new(path: &Path) -> AngularStructure {
        let structure = AngularStructure {
            components: Self::setup_components(path),
            routes: AngularRoutes { value: HashMap::new() },
        };

        structure
    }

    pub fn setup_routes(&mut self, component: &AngularComponent) {
        self.routes = AngularRoutes::new(component, &self.components);
    }

    pub fn get_component_by_kind(&self, kind: ComponentType) -> Vec<AngularComponent> {
        self.components.iter().map(|(_, component)| component).filter(|component| {
            if kind == component.kind {
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
                create_translate_file(structure);

                break;
            }
        };
    }
}