use regex::CaptureMatches;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::io::prelude::*;

lazy_static! {
    static ref TS: Regex = Regex::new(r#"(?m)this\.translate\.instant\(['"`]([\w.${}]*)['"`]"#).unwrap();
    static ref HTML: Regex = Regex::new(r#"(?m)\{\{\s?['|"]([\w|\.]*)['|"]\s?\|\s?translate\s?}}"#).unwrap();
}
lazy_static! {
    static ref C_NAME: Regex = Regex::new(r"(?m)export\sclass\s(.*?)[\s<]").unwrap();
}

#[derive(Clone, Debug, Serialize)]
pub struct TranslationResponse {
    pub components: Vec<AngularComponent>,
    pub routes: HashMap<String, Vec<String>>,
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

    fn register_name(&self) -> String {
        let contents = &self.open_ts();

        C_NAME.captures_iter(&contents)
            .take(1)
            .fold(String::new(), |res, item| item[1].to_string())
    }

    fn get_translations(&self) -> Vec<String> {
        let contents = &self.open_ts();

        // TS extension (default implementation)
        let mut matches: Vec<String> = TS.captures_iter(&contents)
            .into_iter().map(|c| c[1].to_string()).collect();

        // HTML implementation
        if let Some(c) = &self.html {
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