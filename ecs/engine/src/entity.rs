use std::any::TypeId;
use std::boxed::Box;
use std::collections::HashMap;
use std::any::Any;

pub trait Component {
    fn update(&self);
    fn as_any(&self) -> &dyn Any;
}

pub struct Entity {
    id: u64,
    name: String,
    valid: bool,
    components: HashMap<TypeId, Vec<Box<dyn Component>>>,
}

impl Entity {
    pub(crate) fn new(id: u64, name: String) -> Entity {
        Entity {
            id: id,
            name: name,
            valid: true,
            components: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn add_component<T: Component + 'static>(&mut self, new_component: T) -> &T {
        let contains_key = self.components.contains_key(&TypeId::of::<T>());
        if !contains_key {
            self.components.entry(TypeId::of::<T>()).or_insert(vec![]);
        }
        let elem = self.components.get_mut(&TypeId::of::<T>()).unwrap();
        elem.push(Box::new(new_component));
        elem.last().unwrap().as_any().downcast_ref::<T>().unwrap()
    }
    /*
    pub fn get_components<T>(&self) -> &Vec<dyn Component>
    {
        components[TypeId::of::<T>()].as_any().downcast_ref<T>()
    }
    */
}
