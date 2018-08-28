use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::Path;
use utilities::{capture_group, selector_to_component_name, COMPONENTS, ENTRY, C_NAME, TS, HTML};

#[derive(Debug, Serialize)]
pub struct AngularComponents {
    pub value: HashMap<String, AngularComponent>,
}

impl Deref for AngularComponents {
    type Target = HashMap<String, AngularComponent>;

    fn deref(&self) -> &HashMap<String, AngularComponent> {
        &self.value
    }
}

impl AngularComponents {
    pub fn new(path: &Path) -> AngularComponents {
        let mut map = HashMap::new();
        Self::set_components(path, &mut map);

        AngularComponents {
            value: map
        }
    }

    fn set_components(path: &Path, map: &mut HashMap<String, AngularComponent>) {
        let paths = fs::read_dir(path).unwrap();
        let mut map = map;

        for path in paths {
            let entry = path.unwrap();

            if entry.metadata().unwrap().is_dir() {
                let path = entry.path();
                let path = &Path::new(path.as_path());

                Self::set_components(path, &mut map);
            } else {
                if let Some(ex) = entry.path().extension() {
                    if ex == "ts" {
                        let component = AngularComponent::new(entry);
                        map.insert(component.name.clone(), component);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum ComponentType {
    Service,
    Component,
    Directive,
    Module,
    Route,
    Ignore,
}

#[derive(Clone, Debug, Serialize)]
pub struct AngularComponent {
    pub name: String,
    pub kind: ComponentType,
    pub translations: Vec<String>,
    pub file_name: String,
    #[serde(skip)]
    pub path: String,
    #[serde(skip)]
    pub html: Option<String>,
}

impl AngularComponent {
    pub fn new(path: DirEntry) -> AngularComponent {
        let mut component = AngularComponent {
            name: String::new(),
            kind: ComponentType::Ignore,
            translations: Vec::new(),
            file_name: path.file_name().into_string().unwrap(),
            path: path.path().into_os_string().into_string().unwrap(),
            html: None,
        };

        component.name = component.register_name();
        component.kind = component.retrieve_kind();
        component.html = component.find_sibling();
        component.translations = component.get_translations();

        component
    }

    pub fn open_ts(&self) -> String {
        AngularComponent::open(&self.path)
    }

    pub fn open_html(&self) -> String {
        let mut contents = String::new();

        if let Some(c) = &self.html {
            contents = AngularComponent::open(&c);
        }

        contents
    }

    pub fn get_used_components(&self) -> Vec<String> {
        let html = self.open_html();
        let ts = self.open_ts();

        let mut components: Vec<String> = COMPONENTS.captures_iter(&html)
            .into_iter().map(|c| selector_to_component_name(&c[1].to_string())).collect();

        if let Some(s) = capture_group(ENTRY.captures_iter(&ts)) {
            let vec = s.split(",");
            let mut vec: Vec<&str> = vec.collect();

            let vec: Vec<String> = vec.into_iter().map(|s| s.trim_left_matches("\n").trim().to_string()).collect();
            components.extend(vec);
        }

        components
    }

    fn register_name(&self) -> String {
        let contents = &self.open_ts();

        C_NAME.captures_iter(&contents)
            .take(1)
            .fold(String::new(), |_res, item| {
                if let Some(_) = item.get(1) {
                    return item[1].to_string();
                }

                return self.file_name.clone();
            })
    }

    fn get_translations(&self) -> Vec<String> {
        let contents = &self.open_ts();

        // TS extension (default implementation)
        let mut matches: Vec<String> = TS.captures_iter(&contents)
            .into_iter().map(|c| c[1].to_string()).collect();

        // HTML implementation
        if let Some(_c) = &self.html {
            let contents = &self.open_html();
            matches.extend(HTML.captures_iter(&contents)
                .into_iter().map(|c| c[1].to_string()).collect::<Vec<String>>());
        }

        matches
    }

    fn find_sibling(&self) -> Option<String> {
        let vec = &self.path;
        let vec = vec.split(".");
        let mut vec = vec.collect::<Vec<&str>>();

        vec.pop();
        vec.push(&"html");

        let path = vec.join(".");
        if fs::metadata(&path).is_ok() {
            return Some(path);
        }

        None
    }

    fn retrieve_kind(&self) -> ComponentType {
        let vec = &self.file_name;
        let vec = vec.split(".");
        let vec = vec.collect::<Vec<&str>>();

        match &vec[..] {
            [_, "component", _..] => ComponentType::Component,
            [_, "service", _..] => ComponentType::Service,
            [_, "directive", _..] => ComponentType::Directive,
            [_, "module", _..] => ComponentType::Module,
            [_, "routing", _..] => ComponentType::Route,
            _ => ComponentType::Ignore,
        }
    }

    fn open(path: &String) -> String {
        let mut f = File::open(path).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();

        contents
    }
}