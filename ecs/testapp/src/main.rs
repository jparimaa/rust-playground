fn main() {
    let mut scene = engine::scene::Scene::new();

    let dog_entity = scene.create_entity_with_name(String::from("dog"));
    print!("dog_entity id {}\n", dog_entity.borrow().get_id());

    let dog_entity_again = scene.get_entity_by_id(1);
    print!(
        "dog_entity_again id {}\n",
        dog_entity_again
            .as_ref()
            .expect("entity not found")
            .borrow()
            .get_id()
    );

    let unwrapped_dog = dog_entity_again.unwrap();
    {
        print!(
            "unwrapped_dog name {}\n",
            unwrapped_dog.as_ref().borrow().get_name()
        );
    }

    {
        let mut dog_ref = unwrapped_dog.as_ref().borrow_mut();
        dog_ref.set_name(String::from("cat"));
    }

    {
        print!(
            "unwrapped_dog name {}\n",
            unwrapped_dog.as_ref().borrow().get_name()
        );
    }

    let unnamed_entity = scene.create_entity();
    print!(
        "unnamed_entity name {}\n",
        unnamed_entity.borrow().get_name()
    );

    print!("dog_entity is valid: {}\n", dog_entity.borrow().is_valid());
    print!("unwrapped_dog is valid: {}\n", unwrapped_dog.borrow().is_valid());
    let destroyed = scene.destroy_entity(&dog_entity);
    print!("dog_entity was destroyed: {}\n", destroyed);
    print!("dog_entity is valid: {}\n", dog_entity.borrow().is_valid());
    print!("unwrapped_dog is valid: {}", unwrapped_dog.borrow().is_valid());

}
