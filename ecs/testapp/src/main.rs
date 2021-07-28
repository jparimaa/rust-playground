pub mod print_component;

use print_component::PrintComponent;

fn main() {
    let mut scene = engine::scene::Scene::new();

    let dog_entity = scene.create_entity_with_name(String::from("dog"));
    dog_entity.borrow_mut().add_component(PrintComponent::new());
    let cat_entity = scene.create_entity_with_name(String::from("cat"));
    cat_entity
        .borrow_mut()
        .add_component(engine::count_component::CountComponent::new());

    let count_comp = cat_entity
        .borrow_mut()
        .get_component::<engine::count_component::CountComponent>()
        .unwrap();

    let borrowed_comp = count_comp.borrow();

    let count_comp_ref = borrowed_comp
        .as_any()
        .downcast_ref::<engine::count_component::CountComponent>()
        .unwrap();

    loop {
        if count_comp_ref.get_count() > 10 {
            break;
        }

        scene.update();
    }
}
