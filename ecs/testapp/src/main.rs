use std::cell::RefCell;

struct PrintComponent {
    text_to_print: RefCell<String>,
}

impl PrintComponent {
    fn new() -> PrintComponent {
        PrintComponent {
            text_to_print: RefCell::new(String::from("default text\n")),
        }
    }

    fn test_print(&self) {
        print!("test print {}\n", self.text_to_print.borrow());
    }

    fn change_text(&self, new_text: String) {
        *self.text_to_print.borrow_mut() = new_text;
    }
}

impl engine::entity::Component for PrintComponent {
    fn update(&self) {
        print!("update print ");
        self.test_print();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn main() {
    let mut scene = engine::scene::Scene::new();

    let dog_entity = scene.create_entity_with_name(String::from("dog"));
    print!("dog_entity id {}\n", dog_entity.borrow().get_id());

    dog_entity.borrow_mut().add_component(PrintComponent::new());
    let comp = dog_entity
        .borrow_mut()
        .get_component::<PrintComponent>()
        .unwrap();
    comp.borrow().update();

    {
        let _x = comp.borrow_mut();
        let mut _y = _x.as_any().downcast_ref::<PrintComponent>().unwrap();
        _y.test_print();
        _y.change_text(String::from("new text"));
        _y.test_print();
    }

    comp.borrow().update();
}
