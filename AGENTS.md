# Todoz - Rust Desktop App

## Goal

Create a Rust desktop todo app using gpui (the UI framework from Zed). The app should eventually support CRUD operations for todos with keyboard navigation.

## Instructions

- Use gpui from Zed's git repository (latest version with HeadlessAppContext support)
- Start with hardcoded todos
- Build incrementally: list view → keyboard navigation → CRUD
- **Always commit `Cargo.lock`** to ensure reproducible builds
- **After any UI change: run the sway verification workflow (see UI Verification section) before considering the task done. Do NOT rely on `cargo build` success alone.**

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
  - **Calling `window.focus(&focus_handle, cx)` inside `open_window` so keys work immediately without a mouse click**

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

## UI Verification (MANDATORY after any UI change)

After every change that affects rendering, layout, or colors, verify visually using
the headless sway workflow. Do NOT skip this. Do NOT rely on `cargo build` alone.

### Step 1: Teardown any previous instances

```bash
kill $(pgrep -f "target/debug/todoz") 2>/dev/null
kill $(pgrep -x sway) 2>/dev/null
sleep 1
```

### Step 2: Start headless sway

**Must use `WLR_RENDERER=vulkan`** — pixman causes a Mesa DRM fd failure and a fatal crash.

```bash
mkdir -p /tmp/ui-test
WLR_BACKENDS=headless WLR_RENDERER=vulkan XDG_RUNTIME_DIR=/tmp/ui-test \
  sway --config /dev/null > /tmp/ui-test/sway.log 2>&1 &
sleep 2
ls /tmp/ui-test/wayland-1   # socket must exist before continuing
```

### Step 3: Build and launch the app

```bash
cargo build
XDG_RUNTIME_DIR=/tmp/ui-test WAYLAND_DISPLAY=wayland-1 \
  ./target/debug/todoz > /tmp/ui-test/app.log 2>&1 &
sleep 3
```

### Step 4: Capture a screenshot

```bash
XDG_RUNTIME_DIR=/tmp/ui-test WAYLAND_DISPLAY=wayland-1 \
  grim /tmp/ui-test/screenshot.png
```

Read the screenshot file with the Read tool to visually inspect it.

### Step 5: Assert pixel colors with Python/Pillow

```python
from PIL import Image

img = Image.open('/tmp/ui-test/screenshot.png').convert('RGB')

def assert_color(x, y, expected_hex, tolerance=10, label=""):
    r, g, b = img.getpixel((x, y))
    er = (expected_hex >> 16) & 0xff
    eg = (expected_hex >> 8) & 0xff
    eb = expected_hex & 0xff
    assert abs(r - er) <= tolerance, f"{label} R mismatch at ({x},{y}): got {r}, expected {er}"
    assert abs(g - eg) <= tolerance, f"{label} G mismatch at ({x},{y}): got {g}, expected {eg}"
    assert abs(b - eb) <= tolerance, f"{label} B mismatch at ({x},{y}): got {b}, expected {eb}"
    print(f"OK {label} ({x},{y}): #{r:02x}{g:02x}{b:02x}")

print(f"Screenshot size: {img.size}")

# Adapt coordinates based on window position and known layout:
# - 16px outer padding, ~40px row height, 18px circle at row center
assert_color(10, 10, 0x282828, label="background")
# assert_color(x, y, 0x383838, label="selected row bg")
# assert_color(x, y, 0x282828, label="normal row bg")
```

Adapt coordinates by reading `img.size` and reasoning about element positions
from the known layout (16px padding, ~40px row height, etc.).

### Step 6: Verify keyboard navigation

**Note:** `wtype` sends virtual keyboard events into the headless sway session, but GPUI
subscribes to `wl_keyboard` at startup when the headless seat has `capabilities=0` (no
input devices). By the time `wtype` runs, GPUI has already missed the capability
notification and will not receive key events.

**Use the unit test instead** — keyboard navigation is fully covered by `cargo test`:

```bash
cargo test test_keyboard_navigation
```

This test calls `move_down`/`move_up` directly on the view and asserts `selected_index`
changes correctly. It is the authoritative verification for navigation logic.

### Step 7: Teardown

```bash
kill $(pgrep -f "target/debug/todoz") 2>/dev/null
kill $(pgrep -x sway) 2>/dev/null
```

### Current theme colors to assert

| Element | Color |
|---|---|
| App background | `#282828` |
| Normal row background | `#282828` |
| Selected row background | `#383838` |
| Selected row border | `#4a9eff` |
| Circle border (normal) | `#777777` |
| Circle border (focused) | `#4a9eff` |
| Text | `#f0f0f0` |
| Row separator | `#3d3d3d` |

**Keep this table up to date whenever theme colors change.**

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
