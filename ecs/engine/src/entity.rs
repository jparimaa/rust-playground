use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::option::Option;
use std::rc::Rc;

pub trait Component {
    fn update(&self);
    fn as_any(&self) -> &dyn Any;
}

pub struct Entity {
    id: u64,
    name: String,
    valid: bool,
    components: HashMap<TypeId, Vec<Rc<RefCell<dyn Component>>>>,
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

    pub(crate) fn update(&self) {
        for (_, vec) in self.components.iter() {
            for comp in vec.iter() {
                comp.borrow().update()
            }
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

    pub fn add_component<T: Component + 'static>(&mut self, new_component: T) {
        let type_id = TypeId::of::<T>();
        let entry = self.components.entry(type_id).or_insert(vec![]);
        entry.push(Rc::new(RefCell::new(new_component)));
    }

    pub fn get_component<T: Component + 'static>(&mut self) -> Option<Rc<RefCell<dyn Component>>> {
        let type_id = TypeId::of::<T>();
        let entry = self.components.entry(type_id);
        match entry {
            Entry::Occupied(e) => match e.get().first() {
                Some(c) => Option::Some(c.clone()),
                None => Option::None,
            },
            Entry::Vacant(_) => Option::None,
        }
    }

    pub fn get_raw_component<T: Component + 'static>(&mut self) -> Option<*mut T> {
        let type_id = TypeId::of::<T>();
        let entry = self.components.entry(type_id);
        match entry {
            Entry::Occupied(e) => match e.get().first() {
                Some(c) => Option::Some((*c).as_ptr() as *mut T),
                None => Option::None,
            },
            Entry::Vacant(_) => Option::None,
        }
    }

    // Todo: should be moved to component module
    pub fn component_as<T: Component + 'static>(comp: &Rc<RefCell<dyn Component>>) -> &mut T {
        let ptr = (*comp).as_ptr() as *mut T;
        unsafe { &mut *ptr }
    }

    pub fn get_components<T: Component + 'static>(
        &mut self,
    ) -> Option<&Vec<Rc<RefCell<dyn Component>>>> {
        self.components.get(&TypeId::of::<T>())
    }
    // Todo: remove_component
}
