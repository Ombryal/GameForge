use gameforge_ecs::World;
use gameforge_runtime::Runtime;

// Confirms Runtime and World work together, not just individually.
// Each fixed step increments a counter component directly on an entity —
// a stand-in for what a real system will do once systems exist to do
// this instead of an inline closure.
struct TickCount(u32);

#[test]
fn runtime_ticks_drive_component_updates() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, TickCount(0));

    let mut rt = Runtime::new(0.1);
    rt.advance(0.35, |_dt| {
        if let Some(count) = world.get_mut::<TickCount>(entity) {
            count.0 += 1;
        }
    });

    assert_eq!(world.get::<TickCount>(entity).unwrap().0, 3);
    assert_eq!(rt.tick_count(), 3);
}
