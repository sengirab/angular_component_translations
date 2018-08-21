use component::AngularComponent;

#[derive(Debug, Serialize)]
pub struct AngularRoute {
    route: String,
    components: Vec<AngularComponent>,
}

impl AngularRoute {
    pub fn concat_child_routes() {}
}