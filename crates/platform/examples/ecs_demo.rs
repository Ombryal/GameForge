// A small playable loop: walk onto each red square to collect it. Once
// collected an entity is despawned for good, not just hidden, and the
// running score shows in the window title since there's no text
// rendering pipeline to draw it on-canvas yet. Run with:
//   cargo run --example ecs_demo -p gameforge-platform
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
    collectibles: Vec<Entity>,
    total_collectibles: usize,
    score: u32,
}

impl EcsDemo {
    fn new() -> Self {
        let mut world = World::new();

        let player = world.spawn();
        world.insert(player, Position(Vec2::new(100.0, 100.0)));
        world.insert(player, Velocity(Vec2::zero()));

        let collectibles: Vec<Entity> = [
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

        let total_collectibles = collectibles.len();

        Self {
            world,
            player,
            collectibles,
            total_collectibles,
            score: 0,
        }
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

    // Collectibles don't block movement the way the old static obstacles
    // did — touching one collects it instead of stopping the player, so
    // there's nothing left to resolve per-axis here.
    fn movement_system(&mut self, dt: f32) {
        let Some(&Velocity(v)) = self.world.get::<Velocity>(self.player) else {
            return;
        };
        let delta = v.scale(dt);
        if let Some(position) = self.world.get_mut::<Position>(self.player) {
            position.0 = position.0.add(delta);
        }
    }

    // Runs after movement so a collectible reached exactly on the frame
    // the player arrives gets caught immediately instead of one frame
    // late. Despawns from the world outright — a collected item isn't
    // coming back, so there's no reason to keep its component data
    // around with a "collected" flag on it.
    fn collection_system(&mut self) {
        let Some(&Position(player_pos)) = self.world.get::<Position>(self.player) else {
            return;
        };
        let player_rect = Self::rect_at(player_pos);

        let touched: Vec<Entity> = self
            .collectibles
            .iter()
            .copied()
            .filter(|&entity| {
                self.world
                    .get::<Position>(entity)
                    .is_some_and(|&Position(pos)| player_rect.intersects(Self::rect_at(pos)))
            })
            .collect();

        for entity in touched {
            self.world.despawn(entity);
            self.collectibles.retain(|&e| e != entity);
            self.score += 1;
        }
    }
}

impl Frame for EcsDemo {
    fn update(&mut self, dt: f32, input: &InputState) {
        self.input_system(input);
        self.movement_system(dt);
        self.collection_system();
    }

    fn render(&mut self, pixels: &mut [u32], width: u32, height: u32) {
        let mut canvas = Canvas::new(pixels, width, height);
        canvas.clear(0xff181818);

        for &entity in &self.collectibles {
            if let Some(&Position(pos)) = self.world.get::<Position>(entity) {
                canvas.fill_rect(pos.x as i32, pos.y as i32, RECT_SIZE, RECT_SIZE, 0xffff4444);
            }
        }

        if let Some(&Position(pos)) = self.world.get::<Position>(self.player) {
            canvas.fill_rect(pos.x as i32, pos.y as i32, RECT_SIZE, RECT_SIZE, 0xff3388ff);
        }
    }

    fn window_title(&self) -> Option<String> {
        if self.score as usize == self.total_collectibles {
            Some(format!(
                "GameForge - Score: {}/{} - All collected!",
                self.score, self.total_collectibles
            ))
        } else {
            Some(format!(
                "GameForge - Score: {}/{}",
                self.score, self.total_collectibles
            ))
        }
    }
}

fn main() {
    gameforge_platform::run(WindowConfig::default(), EcsDemo::new());
}
