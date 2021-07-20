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
        let name: String = String::from("Entity ") + &(self.last_id.to_string());
        self.entities
            .push(Rc::new(RefCell::new(Entity::new(self.last_id, name))));
        self.entities.last().unwrap().clone()
    }

    pub fn create_entity_with_name(&mut self, name: String) -> Rc<RefCell<Entity>> {
        self.last_id = self.last_id + 1;
        self.entities
            .push(Rc::new(RefCell::new(Entity::new(self.last_id, name))));
        self.entities.last().unwrap().clone()
    }

    pub fn destroy_entity(&mut self, entity: &Rc<RefCell<Entity>>) -> bool {
        let entity_id = entity.borrow().get_id();
        let index = self
            .entities
            .iter()
            .position(|x| x.borrow().get_id() == entity_id);
        match index {
            Some(i) => {
                self.entities[i].borrow_mut().invalidate();
                self.entities.remove(i);
                true
            }
            None => false,
        }
    }

    pub fn get_entity_by_id(&self, id: u64) -> Option<Rc<RefCell<Entity>>> {
        let found = self.entities.iter().find(|x| x.borrow().get_id() == id);
        match found {
            Some(p) => Option::Some(p.clone()),
            None => Option::None,
        }
    }
}
