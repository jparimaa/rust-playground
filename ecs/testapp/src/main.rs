pub mod print_component;

use print_component::PrintComponent;

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

    let print_comp_ref: &print_component::PrintComponent;
    let borrowed_comp = comp.borrow();
    print_comp_ref = borrowed_comp
        .as_any()
        .downcast_ref::<PrintComponent>()
        .unwrap();
    print_comp_ref.test_print();
    print_comp_ref.change_text(String::from("new text"));
    print_comp_ref.test_print();

    comp.borrow().update();

    {
        
        let e1_ref = scene.get_entity_ref_by_id(1).unwrap();
        print!("{}",e1_ref.get_name());
        let e1_ref2 = scene.get_entity_ref_by_id(1).unwrap();
        print!("{}",e1_ref2.get_name());
    }
    
    let e1_mut = scene.get_entity_mut_by_id(1).unwrap();
    print!("{}",e1_mut.get_name());
}
