# Todoz - Rust Desktop App

## Goal

Create a Rust desktop todo app using gpui (the UI framework from Zed). The app should eventually support CRUD operations for todos with keyboard navigation.

## Instructions

- Use gpui from Zed's git repository (latest version with HeadlessAppContext support)
- Start with hardcoded todos
- Build incrementally: list view → keyboard navigation → CRUD

## Dependencies

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed.git", package = "gpui", features = ["test-support"] }
gpui-platform = { git = "https://github.com/zed-industries/zed.git", package = "gpui_platform" }
```

## API Patterns

- Use `gpui_platform::application()` to create app (not `gpui::Application::new()`)
- Use `gpui::Application::with_platform(platform)` for more control
- `div()` is a function, not `Div::new()`
- Need to import traits: `Styled`, `ParentElement`, `AppContext`
- Colors created with `rgb(0xffffff)` function
- Window bounds use `gpui::WindowBounds::Windowed(bounds)` with `gpui::Bounds::centered()`
- **Keyboard handling** requires:
  - Implementing `Focusable` trait with `focus_handle(&self, &App) -> FocusHandle`
  - Using `actions!` macro to define custom actions
  - Binding keys with `cx.bind_keys([KeyBinding::new("up", MoveUp, None), ...])`
  - Using `.track_focus(&focus_handle(cx))` and `.on_action(cx.listener(...))` on the root element
  - Calling `cx.activate(true)` to activate the app

## Testing

### Unit Tests (State-based)

```rust
#[gpui::test]
async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
    let app = cx.add_window(|_window, cx| {
        let focus_handle = cx.focus_handle();
        TodoApp { todos, selected_index: 0, focus_handle }
    });
    _ = app.update(cx, |app, window, cx| {
        app.move_down(&MoveDown, window, cx);
        assert_eq!(app.selected_index, 1);
    });
}
```

### Headless App Context (for screenshots)

The latest gpui has `HeadlessAppContext` with `capture_screenshot()`:

```rust
use gpui::HeadlessAppContext;
use gpui_platform::current_headless_renderer;

let mut cx = HeadlessAppContext::with_platform(
    text_system,
    Arc::new(()),
    || current_headless_renderer(),
);

let window = cx.open_window(size(px(400.0), px(300.0)), |_, cx| {
    cx.new(|_| MyApp { ... })
}).unwrap();

cx.run_until_parked();
let screenshot = cx.capture_screenshot(window.into()).unwrap();
screenshot.save("test.png").unwrap();
```

**Platform Support for Screenshots:**
- macOS: ✓ Full support via Metal headless renderer
- Linux: ✗ Headless renderer not yet implemented

## UI Testing Strategy

### Test Location

Place UI tests in `src/main.rs` within `#[cfg(test)]` module.

### Test Structure

```rust
#[cfg(test)]
mod ui_tests {
    use super::*;
    use gpui::HeadlessAppContext;
    use gpui_platform::current_headless_renderer;
    use std::sync::Arc;
    use image::RgbaImage;

    fn create_test_context() -> HeadlessAppContext {
        let text_system = Arc::new(gpui_linux::LinuxTextSystem::new());
        HeadlessAppContext::with_platform(
            text_system,
            Arc::new(()),
            || current_headless_renderer(),
        )
    }

    fn assert_pixel_color(img: &RgbaImage, x: u32, y: u32, expected: (u8, u8, u8), tolerance: u8) {
        let pixel = img.get_pixel(x, y);
        for (actual, exp) in pixel.0[..3].iter().zip(expected.into_iter()) {
            assert!((actual as i16 - *exp as i16).abs() <= tolerance as i16);
        }
    }
}
```

### Test Categories

1. **Color Verification** - Check specific pixels match expected colors
2. **Layout Verification** - Verify element positions and sizes
3. **State Transition Tests** - Trigger actions, verify visual changes

### Example: Background Color Test

```rust
#[test]
fn test_background_color() {
    let mut cx = create_test_context();

    let window = cx.open_window(
        gpui::size(px(400.0), px(300.0)),
        |_, cx| cx.new(|_| TodoApp { ... })
    ).unwrap();

    cx.run_until_parked();
    let screenshot = cx.capture_screenshot(window.into()).unwrap();

    // Background should be #fdfdfd
    assert_pixel_color(&screenshot, 200, 150, (253, 253, 253), 5);
}
```

### Running UI Tests

```bash
cargo test
```

## Project Structure

```
todoz/
├── Cargo.toml              # gpui dependencies
├── src/
│   └── main.rs            # Main application and tests
```

## Running the App

```bash
cargo run
```

## Running Tests

```bash
cargo test
```

## Next Steps

- Add ability to toggle todo completion (spacebar or similar)
- Add new todo creation
- Delete todos
- Persist todos to storage
- Add better styling
- Implement headless screenshot for Linux (requires PlatformHeadlessRenderer implementation)
