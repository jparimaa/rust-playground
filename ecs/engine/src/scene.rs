use crate::entity::Entity;
use std::cell::RefCell;
use std::option::Option;
use std::rc::Rc;

pub struct Scene {
    entities: Vec<Rc<RefCell<Entity>>>,
    last_id: u64,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            entities: vec![],
            last_id: 0,
        }
    }

    pub fn create_entity(&mut self) -> Rc<RefCell<Entity>> {
        self.last_id = self.last_id + 1;
        self.entities.push(Rc::new(RefCell::new(Entity::new(
            self.last_id,
            String::from("test"),
        ))));
        self.entities.last().unwrap().clone()
    }

    pub fn get_entity_by_id(&self, id: u64) -> Option<Rc<RefCell<Entity>>> {
        let found = self.entities.iter().find(|x| x.borrow().get_id() == id);
        match found {
            Some(p) => Option::Some(p.clone()),
            None => Option::None,
        }
    }
}
