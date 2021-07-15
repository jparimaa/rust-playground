fn main() {
    let mut scene = engine::scene::Scene::new();
    let entity = scene.create_entity();
    entity.borrow().get_id();
    print!("entity id {}\n", entity.borrow().get_id());

    let entity2 = scene.get_entity_by_id(1);    
    print!("entity id {}\n", entity2.expect("entity not found").borrow().get_id());

}
