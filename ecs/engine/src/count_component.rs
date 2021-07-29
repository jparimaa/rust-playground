pub struct CountComponent {
    count: std::cell::Cell<u64>,
}

impl CountComponent {
    pub fn new() -> CountComponent {
        CountComponent {
            count: std::cell::Cell::new(0),
        }
    }

    pub fn get_count(&self) -> u64 {
        self.count.get()
    }

    pub fn override_count(&mut self, count: u64) {
        self.count.set(count);
    }
}

impl crate::entity::Component for CountComponent {
    fn update(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
