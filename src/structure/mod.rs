use structure::route::AngularRoutes;
use structure::component::AngularComponents;
use std::collections::HashMap;
use structure::component::AngularComponent;
use structure::component::ComponentType;
use std::path::Path;

pub mod component;
pub mod route;
mod utilities;

#[derive(Debug, Serialize)]
pub struct AngularStructure {
    pub routes: AngularRoutes,
    pub components: AngularComponents,
}

impl AngularStructure {
    pub fn new<P: AsRef<Path>>(path: P) -> AngularStructure {
        let structure = AngularStructure {
            components: Self::setup_components(path.as_ref()),
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
}
