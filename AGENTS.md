# Todoz - Rust Desktop App

## Goal

Create a Rust desktop todo app using gpui (the UI framework from Zed). The app should eventually support CRUD operations for todos with keyboard navigation.

## Instructions

- Use gpui 0.2.2 from crates.io as the UI framework
- Start with hardcoded todos
- Build incrementally: list view → keyboard navigation → CRUD

## Discoveries

- gpui 0.2.2 has a significantly different API than the Zed internal version used in online examples
- Key API patterns found:
  - `div()` is a function, not `Div::new()`
  - Use `gpui::Application::new().run()` pattern
  - Need to import traits: `Styled`, `ParentElement`, `AppContext`
  - Colors created with `rgb(0xffffff)` function
  - Window bounds use `gpui::WindowBounds::Windowed(bounds)` with `gpui::Bounds::centered()`
- **Keyboard handling** requires:
  - Implementing `Focusable` trait with `focus_handle(&self, &App) -> FocusHandle`
  - Using `actions!` macro to define custom actions
  - Binding keys with `app.bind_keys([KeyBinding::new("up", MoveUp, None), ...])`
  - Using `.track_focus(&focus_handle)` and `.on_action(cx.listener(...))` on the root element
  - Calling `window.focus(&focus_handle)` after window creation

## Project Structure

```
todoz/
├── Cargo.toml              # gpui dependency configured
├── src/
│   └── main.rs            # Main application code with todo list and keyboard navigation
```

## Running the App

```bash
cargo run
```

## Testing

When testing GUI features:
1. Describe what to test (which keys to press, expected behavior)
2. Run the app with `cargo run`
3. Wait for user to interact with it
4. User reports if it works as expected

## Next Steps

- Add ability to toggle todo completion (spacebar or similar)
- Add new todo creation
- Delete todos
- Persist todos to storage
- Add better styling
