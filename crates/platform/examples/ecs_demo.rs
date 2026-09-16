// Proves ecs and platform actually compose: a World holds a player
// entity (Position + Velocity) and three static obstacles (Position
// only), and movement_system only touches the ones with a Velocity to
// move, skipping the rest without needing to know they exist. Run with:
//   cargo run --example ecs_demo -p gameforge-platform
// WASD/arrows move the blue square around the three static red squares.
use gameforge_ecs::{Entity, World};
use gameforge_math::Vec2;
use gameforge_platform::{Frame, InputState, WindowConfig};
use gameforge_renderer2d::Canvas;
use winit::keyboard::KeyCode;

const RECT_SIZE: u32 = 40;
const MOVE_SPEED: f32 = 240.0;

struct Position(Vec2);
struct Velocity(Vec2);

struct EcsDemo {
    world: World,
    player: Entity,
    obstacles: Vec<Entity>,
}

impl EcsDemo {
    fn new() -> Self {
        let mut world = World::new();

        let player = world.spawn();
        world.insert(player, Position(Vec2::new(100.0, 100.0)));
        world.insert(player, Velocity(Vec2::zero()));

        let obstacles = [
            Vec2::new(400.0, 200.0),
            Vec2::new(600.0, 400.0),
            Vec2::new(300.0, 500.0),
        ]
        .into_iter()
        .map(|pos| {
            let e = world.spawn();
            world.insert(e, Position(pos));
            e
        })
        .collect();

        Self { world, player, obstacles }
    }

    // Obstacles have no Velocity component, so there's nothing here for
    // them to opt into — this only ever touches the player.
    fn input_system(&mut self, input: &InputState) {
        let mut dir = Vec2::zero();
        if input.is_held(KeyCode::KeyW) || input.is_held(KeyCode::ArrowUp) {
            dir.y -= 1.0;
        }
        if input.is_held(KeyCode::KeyS) || input.is_held(KeyCode::ArrowDown) {
            dir.y += 1.0;
        }
        if input.is_held(KeyCode::KeyA) || input.is_held(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if input.is_held(KeyCode::KeyD) || input.is_held(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        if let Some(velocity) = self.world.get_mut::<Velocity>(self.player) {
            velocity.0 = dir.normalized().scale(MOVE_SPEED);
        }
    }

    // Collected into an owned Vec first: iterating world.entities()
    // directly while calling world.get_mut() inside the loop body would
    // hold an immutable borrow of world alive across a mutable one.
    // Cloning three entity ids is free; a real query API would avoid
    // needing this at all once one exists.
    fn movement_system(&mut self, dt: f32) {
        for entity in self.world.entities().to_vec() {
            let Some(Velocity(v)) = self.world.get::<Velocity>(entity) else {
                continue;
            };
            let delta = v.scale(dt);
            if let Some(position) = self.world.get_mut::<Position>(entity) {
                position.0 = position.0.add(delta);
            }
        }
    }
}

impl Frame for EcsDemo {
    fn update(&mut self, dt: f32, input: &InputState) {
        self.input_system(input);
        self.movement_system(dt);
    }

    fn render(&mut self, pixels: &mut [u32], width: u32, height: u32) {
        let mut canvas = Canvas::new(pixels, width, height);
        canvas.clear(0xff181818);

        for &obstacle in &self.obstacles {
            if let Some(Position(pos)) = self.world.get::<Position>(obstacle) {
                canvas.fill_rect(pos.x as i32, pos.y as i32, RECT_SIZE, RECT_SIZE, 0xffff4444);
            }
        }

        if let Some(Position(pos)) = self.world.get::<Position>(self.player) {
            canvas.fill_rect(pos.x as i32, pos.y as i32, RECT_SIZE, RECT_SIZE, 0xff3388ff);
        }
    }
}

fn main() {
    gameforge_platform::run(WindowConfig::default(), EcsDemo::new());
}
