// Manual smoke test — opens a real OS window, so it can't run headlessly
// in CI the way the other crates' tests do. `cargo run --example
// open_window -p gameforge-platform` and confirm a window opens and
// closes cleanly.
fn main() {
    gameforge_platform::run(gameforge_platform::WindowConfig::default());
}
