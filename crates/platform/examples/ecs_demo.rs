// Proves ecs, math, and platform compose into an actual tiny playable
// demo: a World holds a player entity (Position + Velocity) and three
// static obstacles (Position only). movement_system moves the player,
// then stops it at whichever axis would have overlapped an obstacle —
// obstacles never move and never need a Velocity to block the player.
// Run with:
//   cargo run --example ecs_demo -p gameforge-platform
// WASD/arrows move the blue square; it should stop at the edges of the
// three red squares instead of passing through them.
use gameforge_ecs::{Entity, World};
use gameforge_math::{Rect, Vec2};
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

    fn rect_at(pos: Vec2) -> Rect {
        Rect::new(pos.x, pos.y, RECT_SIZE as f32, RECT_SIZE as f32)
    }

    // Moves each axis separately and checks collision after each one,
    // rather than moving both axes then checking once. Combined-axis
    // movement would either block a diagonal step entirely when only one
    // axis actually hit something, or require deciding which axis "wins"
    // — resolving per-axis sidesteps both problems and is what lets the
    // player slide along an obstacle's edge instead of stopping dead the
    // moment either axis would collide.
    fn movement_system(&mut self, dt: f32) {
        for entity in self.world.entities().to_vec() {
            let Some(Velocity(v)) = self.world.get::<Velocity>(entity) else {
                continue;
            };
            let delta = v.scale(dt);
            let Some(&Position(current)) = self.world.get::<Position>(entity) else {
                continue;
            };

            let mut next = current;

            let stepped_x = Vec2::new(next.x + delta.x, next.y);
            if !self.hits_any_obstacle(entity, stepped_x) {
                next = stepped_x;
            }

            let stepped_y = Vec2::new(next.x, next.y + delta.y);
            if !self.hits_any_obstacle(entity, stepped_y) {
                next = stepped_y;
            }

            if let Some(position) = self.world.get_mut::<Position>(entity) {
                position.0 = next;
            }
        }
    }

    fn hits_any_obstacle(&self, moving: Entity, at: Vec2) -> bool {
        let candidate = Self::rect_at(at);
        self.obstacles.iter().any(|&obstacle| {
            obstacle != moving
                && self
                    .world
                    .get::<Position>(obstacle)
                    .is_some_and(|&Position(pos)| candidate.intersects(Self::rect_at(pos)))
        })
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
