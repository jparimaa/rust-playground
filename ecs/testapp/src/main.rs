fn main() {
    println!("Hello, world!");
    let _e = engine::Entity::new(0, String::from("test"));
    
    let mut s = engine::Scene::new();    
    let _entity = s.create_entity();
    
}
