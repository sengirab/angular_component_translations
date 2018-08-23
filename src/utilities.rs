extern crate serde_json;

use component::{AngularComponent, ComponentType, TranslationResponse};
use regex::CaptureMatches;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

lazy_static! {
    static ref RE_TS: Regex = Regex::new(r#"(?m)this\.translate\.instant\(['"`]([\w.${}]*)['"`]"#).unwrap();
    static ref RE_HTML: Regex = Regex::new(r#"(?m)\{\{\s?['|"]([\w|\.]*)['|"]\s?\|\s?translate\s?}}"#).unwrap();
}

pub fn create_translate_file(mut translations: TranslationResponse) {
    translations.components = remove_empty_and_ignored(translations.components);

    let json = serde_json::to_string(&translations).unwrap();
    let mut file = File::create("component_translation_keys.json")
        .expect("Failed creating file.");

    file.write(json.into_bytes().as_slice())
        .expect("Failed writing file");
}

pub fn return_components(path: &Path, vec: Vec<AngularComponent>) -> Vec<AngularComponent> {
    let paths = fs::read_dir(path).unwrap();
    let mut vec = vec;

    for path in paths {
        match path {
            Ok(entry) => {
                if entry.metadata().unwrap().is_dir() {
                    let path = entry.path();
                    let path = &Path::new(path.as_path());

                    vec = return_components(path, vec);
                } else {
                    if let Some(ex) = entry.path().extension() {
                        if ex == "ts" {
                            vec.push(AngularComponent::new(entry));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    vec
}

pub fn remove_empty_and_ignored(vec: Vec<AngularComponent>) -> Vec<AngularComponent> {
    vec.into_iter().filter(|component| {
        if component.kind == ComponentType::Ignore || component.translations.is_empty() {
            return false;
        }

        true
    }).collect()
}

pub fn replace_extension(file_name: &String, replace: &str) -> String {
    let vec = file_name.split(".");
    let mut vec = vec.collect::<Vec<&str>>();

    vec.pop();
    vec.push(replace);

    vec.join(".")
}
