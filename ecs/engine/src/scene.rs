use crate::entity::Entity;

pub struct Scene {
    entities: Vec<Entity>,
    last_id: u64,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            entities: Vec::new(),
            last_id: 1,
        }
    }

    pub fn create_entity(&mut self) -> &Entity {
        self.last_id = self.last_id + 1;
        self.entities
            .push(Entity::new(self.last_id, String::from("test")));
        self.entities.last().unwrap()
    }
}
