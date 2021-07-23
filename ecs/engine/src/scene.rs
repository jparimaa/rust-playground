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

    pub fn update(&self) {
        for entity in self.entities.iter() {
            entity.borrow().update();
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

    pub fn get_entity_by_name(&self, name: String) -> Option<Rc<RefCell<Entity>>> {
        let found = self.entities.iter().find(|x| x.borrow().get_name() == name);
        match found {
            Some(p) => Option::Some(p.clone()),
            None => Option::None,
        }
    }

    pub fn get_entities(&self) -> &Vec<Rc<RefCell<Entity>>> {
        &self.entities
    }

    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entity() {
        let mut scene = Scene::new();
        let entity_count = 5;
        for _ in 0..entity_count {
            scene.create_entity();
        }
        assert_eq!(entity_count, scene.get_entity_count());
        assert_eq!(entity_count, scene.get_entities().len());
    }

    #[test]
    fn create_entity_with_name() {
        let mut scene = Scene::new();
        let dog_name = String::from("dog");
        let cat_name = String::from("cat");
        scene.create_entity_with_name(dog_name.clone());
        scene.create_entity_with_name(cat_name.clone());
        scene.get_entity_by_name(dog_name).unwrap();
        scene.get_entity_by_name(cat_name).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_nonexisting_entity_by_name() {
        let scene = Scene::new();
        scene.get_entity_by_name(String::from("mouse")).unwrap();
    }

    #[test]
    fn destroy_entity_with_name() {
        let mut scene = Scene::new();
        assert_eq!(0, scene.get_entity_count());
        let dog_entity = scene.create_entity_with_name(String::from("dog"));
        assert_eq!(1, scene.get_entity_count());
        let cat_entity = scene.create_entity_with_name(String::from("cat"));
        assert_eq!(2, scene.get_entity_count());
        scene.destroy_entity(&dog_entity);
        assert_eq!(1, scene.get_entity_count());
        scene.destroy_entity(&cat_entity);
        assert_eq!(0, scene.get_entity_count());
    }
}
