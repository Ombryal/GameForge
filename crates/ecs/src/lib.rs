use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

// One HashMap per component type, keyed by entity. Simple to reason
// about and fast enough at current entity counts. A sparse-set or
// archetype layout can replace this later if profiling says it matters.
#[derive(Default)]
pub struct World {
    next_id: u32,
    entities: Vec<Entity>,
    components: HashMap<TypeId, HashMap<Entity, Box<dyn Any>>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        self.entities.push(entity);
        entity
    }

    // Components aren't indexed by entity beyond their own per-type map,
    // so this walks every component type to clean one entity out. Fine
    // at current scale; revisit if the type count grows large.
    pub fn despawn(&mut self, entity: Entity) {
        self.entities.retain(|&e| e != entity);
        for storage in self.components.values_mut() {
            storage.remove(&entity);
        }
    }

    pub fn insert<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(HashMap::new)
            .insert(entity, Box::new(component));
    }

    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.get(&entity))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|storage| storage.get_mut(&entity))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }

    #[test]
    fn spawn_creates_distinct_entities() {
        let mut world = World::new();
        let a = world.spawn();
        let b = world.spawn();
        assert_ne!(a, b);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn insert_and_get_round_trips_a_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn despawn_removes_components_too() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.despawn(e);
        assert!(world.get::<Position>(e).is_none());
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn get_mut_allows_updating_a_component_in_place() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.get_mut::<Position>(e).unwrap().x = 5.0;
        assert_eq!(world.get::<Position>(e).unwrap().x, 5.0);
    }
}
