use component::AngularComponent;
use std::collections::HashMap;
use std::ops::Deref;
use std::collections::HashSet;
use utilities::capture_group;
use pcre::Pcre;
use utilities::replace_extension;

#[derive(Debug, Serialize)]
pub struct AngularRoutes {
    value: HashMap<String, Vec<String>>,
}

impl Deref for AngularRoutes {
    type Target = HashMap<String, Vec<String>>;

    fn deref(&self) -> &HashMap<String, Vec<String>> {
        &self.value
    }
}

impl AngularRoutes {
    pub fn new(component: &AngularComponent) -> AngularRoutes {
        let content = component.open_ts();
        let routes = capture_group(ROUTES.captures_iter(&content));

        let mut angular_routes = AngularRoutes {
            value: HashMap::new()
        };

        angular_routes.setup_route_hierarchy(&routes.unwrap(), &String::new(), None);

        // Drain map of all duplicates
        map = map.into_iter().map(|(k, mut v)| {
            let set: HashSet<_> = v.drain(..).collect();
            v.extend(set.into_iter());

            (k, v)
        }).collect();

        map
    }

    fn find_components(&self, name: &str, found: &mut Vec<String>) {
        let component = self.components.get(name).unwrap();
        let components = component.get_used_components();

        for component in &components {
            found.push(component.clone());
            self.find_components(component, found);
        }
    }

    fn setup_route_hierarchy(&mut self, routes: &String, path: &String, main_components: Option<Vec<String>>) {
        let mut re = Pcre::compile(r"(?m)(\{[^}\{]*(?:(?R)[^}{]*)*+\})").unwrap();
        let matches = re.matches(routes);

        for capture in matches {
            let group: &str = capture.group(1);
            // Match all different types in here
            let set = RegexSet::new(&[PATH.as_str(), COMPONENT.as_str(), LOAD.as_str(), CHILDREN.as_str()]).unwrap();
            let set: Vec<_> = set.matches(group).into_iter().collect();

            // Clone path for each route found. We want a full path PER main route,
            let mut path = path.clone();
            let mut main_components: Vec<String> = Vec::new();
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
                        find_components(&matches, &mut found);

                        let mut components = self.entry(path.clone()).or_insert(Vec::new());
                        components.extend(found);
                        if let Some(c) = main_components.clone() {
                            components.extend(c);
                        }

                        main_components = components.clone();
                    }
                    // Find file that's being loaded and go recursive
                    2 => {
                        let matches = capture_group(LOAD.captures_iter(group));

                        if let Some(c) = capture_group(LOAD.captures_iter(group)) {
                            let file_name = replace_extension(&c, "routing.ts");
                            let state = STATE.read().unwrap();
                            let component = state.get(&file_name).unwrap();

                            let routes = capture_group(ROUTES.captures_iter(&component.open_ts()));

                            self.setup_route_hierarchy(&routes.unwrap(), &path,None);
                        }
                    }
                    // Go recursive with matches.
                    3 => {
                        let matches = capture_group(CHILDREN.captures_iter(group));
                        self.setup_route_hierarchy(&matches.unwrap(), &path,Some(main_components.clone()));
                    }
                    _ => {}
                }
            }
        }
    }
}