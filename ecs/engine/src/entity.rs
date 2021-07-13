pub struct Entity {
    id: u64,
    name: String,
}

impl Entity {
    pub fn new(id: u64, name: String) -> Entity {
        Entity { id: id, name: name }
    }
}
