#[test]
fn add_count_component() {
    let mut scene = engine::scene::Scene::new();
    let entity = scene.create_entity();
    
    entity
        .borrow_mut()
        .add_component(engine::count_component::CountComponent::new());

    scene.update();
    scene.update();
    scene.update();

    let comp = entity
        .borrow_mut()
        .get_component::<engine::count_component::CountComponent>()
        .unwrap();

    let comp_borrow = comp.borrow();
    let count_comp = comp_borrow
        .as_any()
        .downcast_ref::<engine::count_component::CountComponent>()
        .unwrap();
    assert_eq!(count_comp.get_count(), 3);
}
