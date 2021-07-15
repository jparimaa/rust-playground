pub struct Entity {
    id: u64,
    name: String,
}

impl Entity {
    pub fn new(id: u64, name: String) -> Entity {
        Entity { id: id, name: name }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
