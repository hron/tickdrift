# Tickdrift - Rust Desktop App

## Goal

Create a Rust desktop todo app using gpui (the UI framework from Zed). The app should eventually support CRUD operations for todos with keyboard navigation.

## Instructions

- Use gpui from Zed's git repository (latest version with HeadlessAppContext support)
- Start with hardcoded todos
- Build incrementally: list view → keyboard navigation → CRUD
- **Always commit `Cargo.lock`** to ensure reproducible builds
- **Always run `cargo fmt` after applying any edits to `.rs` files.**

## Dependencies

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed.git", package = "gpui", features = ["test-support"] }
gpui-platform = { git = "https://github.com/zed-industries/zed.git", package = "gpui_platform" }
```

## Colors — Never Hardcode

**Always use `cx.theme()` fields from gpui-component's `ThemeColor`. Never use `rgb()`, `hsla()`, or hex literals for UI colors.**

Pick the field that is semantically appropriate for the element's meaning:

| `cx.theme()` field | Semantic meaning | Use for |
|---|---|---|
| `.background` | App/window background | Window fill, inner circle punch-out |
| `.foreground` | Primary text | Body text |
| `.muted` | Subtle background | Separators, dividers |
| `.muted_foreground` | Disabled / low-emphasis | P4 circle, completed text, completed circle |
| `.primary` | Brand / action | Selection border, focus rings |
| `.danger` | Destructive / urgent (red) | P1 circle |
| `.warning` | High priority / alert (yellow-orange) | P2 circle |
| `.info` | Informational / medium (cyan-blue) | P3 circle |
| `.success` | Positive outcome (green) | (reserved for future use) |
| `.border` | Default border | Generic outlines |
| `.ring` | Focus ring | Keyboard focus outlines |

Full `ThemeColor` field list lives in the gpui-component source at:
`crates/ui/src/theme/theme_color.rs`

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

### Visual Verification (real app, screenshots, input)

Unit tests check state, not pixels. After any UI change, also check the real app with
`scripts/ui`. It runs tickdrift in an isolated headless Wayland compositor (`cage`), exposes it
over VNC (`wayvnc`, `127.0.0.1:5900`) and drives it with `vncdo`. The user's desktop session is
not affected.

```bash
scripts/ui up              # start cage + wayvnc (idempotent)
scripts/ui run             # cargo build + (re)start the app
scripts/ui shot <name>     # -> target/ui/shots/<name>.png, then open it with the Read tool
scripts/ui key down ctrl-e # vncdotool key names: down, enter, tab, esc, ctrl-e, ...
scripts/ui type "text"
scripts/ui click X Y       # screen coordinates as seen on the screenshot
scripts/ui logs            # app / cage / wayvnc logs
scripts/ui down
```

- Call `scripts/ui` as a plain command (no `cd … &&` prefix). It is excluded from the Claude Code
  sandbox via `excludedCommands: ["scripts/ui *"]`, because Wayland needs Unix sockets.
- The app is fullscreen 1280x720 inside cage. Rendering uses the Mesa lavapipe software Vulkan driver.
- Host packages: `cage`, `wayvnc`, `grim`, `vulkan-swrast`, `vncdotool`.
- Humans can watch live with any VNC client at `127.0.0.1:5900`.

## Project Structure

```
tickdrift/
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
