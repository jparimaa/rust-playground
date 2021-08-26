use std::cell::RefCell;

pub struct PrintComponent {
    text_to_print: RefCell<String>,
}

impl PrintComponent {
    pub fn new() -> PrintComponent {
        PrintComponent {
            text_to_print: RefCell::new(String::from("default text\n")),
        }
    }

    pub fn test_print(&self) {
        print!("test print {}\n", self.text_to_print.borrow());
    }

    pub fn change_text(&self, new_text: String) {
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
