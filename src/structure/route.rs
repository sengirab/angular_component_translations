use pcre::Pcre;
use regex::RegexSet;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use structure::component::AngularComponent;
use structure::component::AngularComponents;
use structure::utilities::{capture_group, CHILDREN, COMPONENT, LOAD, PATH, replace_extension, ROUTES};
use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct AngularRoutes {
    pub value: HashMap<String, Vec<String>>,
}

impl Deref for AngularRoutes {
    type Target = HashMap<String, Vec<String>>;

    fn deref(&self) -> &HashMap<String, Vec<String>> {
        &self.value
    }
}

impl DerefMut for AngularRoutes {
    fn deref_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.value
    }
}

impl AngularRoutes {
    pub fn new(component: &AngularComponent, components: &AngularComponents) -> AngularRoutes {
        let content = component.open_ts();
        let content = capture_group(ROUTES.captures_iter(&content));

        let mut routes = AngularRoutes {
            value: HashMap::new()
        };

        routes.setup_route_hierarchy(&content.unwrap(), &String::new(), None, components);
        routes.dedup();

        routes
    }

    fn dedup(&mut self) {
        let value: HashMap<String, Vec<String>> = self.clone().into_iter().map(|(k, mut v)| {
            let set: HashSet<_> = v.drain(..).collect();
            v.extend(set.into_iter());

            (k, v)
        }).collect();

        self.value = value;
    }


    fn find_components(&self, name: &str, found: &mut Vec<String>, components: &AngularComponents) {
        let component = components.get(name).unwrap();
        let used_components = component.get_used_components();

        for component in &used_components {
            found.push(component.clone());
            self.find_components(component, found, &components);
        }
    }

    fn setup_route_hierarchy(&mut self, routes: &String, path: &String, main_components: Option<Vec<String>>, components: &AngularComponents) {
        let mut re = Pcre::compile(r"(?m)(\{[^}\{]*(?:(?R)[^}{]*)*+\})").unwrap();
        let matches = re.matches(routes);

        for capture in matches {
            let group: &str = capture.group(1);
            // Match all different types in here
            let set = RegexSet::new(&[PATH.as_str(), COMPONENT.as_str(), LOAD.as_str(), CHILDREN.as_str()]).unwrap();
            let set: Vec<_> = set.matches(group).into_iter().collect();

            // Clone path for each route found. We want a full path PER main route,
            let mut path = path.clone();
            let mut previous_components: Vec<String> = Vec::new();
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
                        let matches = matches.unwrap();

                        let mut found = vec![matches.clone()];
                        self.find_components(&matches, &mut found, &components);

                        let mut route_components = self.entry(path.clone()).or_insert(Vec::new());

                        route_components.extend(found);
                        if let Some(c) = main_components.clone() {
                            route_components.extend(c);
                        }

                        previous_components = route_components.clone().to_vec();
                    }
                    // Find file that's being loaded and go recursive
                    2 => {
                        if let Some(c) = capture_group(LOAD.captures_iter(group)) {
                            let file_name = replace_extension(&c, "routing.ts");
                            let component = components.get(&file_name).unwrap();

                            let routes = capture_group(ROUTES.captures_iter(&component.open_ts()));
                            self.setup_route_hierarchy(&routes.unwrap(), &path, None, &components);
                        }
                    }
                    // Go recursive with matches.
                    3 => {
                        let matches = capture_group(CHILDREN.captures_iter(group));
                        self.setup_route_hierarchy(&matches.unwrap(), &path, Some(previous_components.clone()), &components);
                    }
                    _ => {}
                }
            }
        }
    }
}