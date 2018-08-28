extern crate serde_json;

use regex::{CaptureMatches, Regex};

lazy_static! {
    pub static ref ROUTES: Regex = Regex::new(r"(?m)Routes\s?=\s?(\[\s[^;]*);").unwrap();
    pub static ref PATH: Regex = Regex::new(r#"(?m)path:\s['"`](.*?)['"`]"#).unwrap();
    pub static ref COMPONENT: Regex = Regex::new(r"(?m)component:\s?(\w+)").unwrap();
    pub static ref LOAD: Regex = Regex::new(r"(?mis)(?:\schildren.*#)|(\w+\.\w+)#").unwrap();
    pub static ref CHILDREN: Regex = Regex::new(r"(?ms)children: \[(.*?)^[[:blank:]]{8}]").unwrap();
    pub static ref COMPONENTS: Regex = Regex::new(r"(?m)<(app-(?:\w+-?)*)").unwrap();
    pub static ref ENTRY: Regex = Regex::new(r"(?sm)entryComponents: \[\s*(.*)]").unwrap();
    pub static ref TS: Regex = Regex::new(r#"(?m)this\.translate\.instant\(['"`]([\w.${}]*)['"`]"#).unwrap();
    pub static ref HTML: Regex = Regex::new(r#"(?m)\{\{\s?['|"]([\w|\.]*)['|"]\s?\|\s?translate\s?}}"#).unwrap();
    pub static ref C_NAME: Regex = Regex::new(r"(?m)export\sclass\s(.*?)[\s<]|const\s(.*):\s?Routes").unwrap();
}

pub fn replace_extension(file_name: &String, replace: &str) -> String {
    let vec = file_name.split(".");
    let mut vec = vec.collect::<Vec<&str>>();

    vec.pop();
    vec.push(replace);

    vec.join(".")
}

pub fn capture_group(captures: CaptureMatches) -> Option<String> {
    captures
        .take(1)
        .fold(None, |_res, item| {
            if let Some(_) = item.get(1) {
                return Some(item[1].to_string());
            }

            None
        })
}

pub fn selector_to_component_name(name: &str) -> String {
    let vec = name.split("-");
    let mut vec: Vec<&str> = vec.collect();

    vec.remove(0);
    let mut vec: String = vec.iter().map(|s| uppercase_first_letter(s)).collect();
    vec.push_str("Component");

    vec
}

pub fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
