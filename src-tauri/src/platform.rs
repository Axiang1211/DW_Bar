//! Taskbar placement glue. All functions run on the UI thread.
//! Off Windows these are no-ops, so the UI can still be developed on any OS
//! (the capsule then shows as a small floating window).
use crate::state::AppState;
use tauri::{AppHandle, Manager, WebviewWindow};

#[cfg(windows)]
fn hwnd_of(win: &WebviewWindow) -> Option<isize> {
    win.hwnd().ok().map(|h| h.0 as isize)
}

/// `lint-win`: lets the Windows branches be type-checked on another OS.
#[cfg(all(not(windows), feature = "lint-win"))]
fn hwnd_of(_win: &WebviewWindow) -> Option<isize> {
    None
}

/// Call once right after the capsule window is created.
pub fn prepare(win: &WebviewWindow) {
    #[cfg(windows)]
    if let Some(hwnd) = hwnd_of(win) {
        dwbar_core::win::embed::prepare(hwnd);
    }
    #[cfg(not(windows))]
    let _ = win;
}

/// Attach, size, place and show/hide the capsule according to the config.
pub fn apply(app: &AppHandle) {
    let Some(win) = app.get_webview_window(crate::CAPSULE) else { return };
    let state = app.state::<AppState>();

    #[cfg(any(windows, feature = "lint-win"))]
    {
        use crate::state::Placement;
        use dwbar_core::{config::{Attach, Position}, layout, win::{embed, taskbar}};
        let Some(hwnd) = hwnd_of(&win) else { return };
        let (config, width_logical, height_logical, fullscreen) = {
            let s = state.inner.lock().unwrap();
            (s.config.clone(), s.capsule_width, s.capsule_height, s.fullscreen)
        };
        let Some((tb, info)) = taskbar::info() else {
            // Explorer is restarting; TaskbarCreated will bring us back.
            embed::set_visible(hwnd, false);
            return;
        };
        let floating = config.position != Position::Taskbar;
        let work = taskbar::work_area(tb);

        let (rect, embedded) = if floating {
            // Floating HUD: a top-most tool window at the chosen edge / corner.
            let thick = (layout::HUD_THICKNESS * info.scale).round() as i32;
            let (w, h) = if config.position.vertical() {
                (thick, ((height_logical * info.scale).round() as i32).max(thick))
            } else {
                (((width_logical * info.scale).round() as i32).max(thick), thick)
            };
            let area = work.unwrap_or(info.rect);
            if embed::is_child_of(hwnd, tb) {
                embed::unembed(hwnd);
            }
            let rect = layout::place_floating(&area, config.position, info.scale, w, h);
            embed::place(hwnd, rect, None);
            (rect, false)
        } else {
            let height = layout::capsule_height(&info);
            let width = (width_logical * info.scale).round().max(height as f64) as i32;
            let rect = layout::place(&info, config.slot, config.offset_x, width, height);
            let embedded = match config.attach {
                Attach::Embed => embed::embed(hwnd, tb),
                Attach::Overlay => false,
            };
            if !embedded && embed::is_child_of(hwnd, tb) {
                embed::unembed(hwnd);
            }
            embed::place(hwnd, rect, embedded.then_some(tb));
            (rect, embedded)
        };
        let visible = !(config.hide_fullscreen && fullscreen);
        embed::set_visible(hwnd, visible);

        let mut s = state.inner.lock().unwrap();
        s.placement = Placement {
            found_taskbar: true,
            embedded,
            fallback: !floating && config.attach == Attach::Embed && !embedded,
            visible,
            scale: info.scale,
            centered: info.centered,
            widgets: info.widgets,
            taskbar: Some(info.rect),
            capsule: Some(rect),
            work,
            floating,
        };
    }

    #[cfg(not(any(windows, feature = "lint-win")))]
    {
        let s = state.inner.lock().unwrap();
        let _ = win.set_size(tauri::LogicalSize::new(s.capsule_width, 40.0));
        let _ = win.show();
    }
}

/// Cheap periodic upkeep: keep z-order above the taskbar content and follow
/// geometry changes (taskbar resized, DPI changed, alignment toggled).
#[cfg_attr(not(any(windows, feature = "lint-win")), allow(dead_code))]
pub fn tick(app: &AppHandle) {
    #[cfg(any(windows, feature = "lint-win"))]
    {
        use dwbar_core::win::{embed, taskbar};
        let Some(win) = app.get_webview_window(crate::CAPSULE) else { return };
        let Some(hwnd) = hwnd_of(&win) else { return };
        let (last, embedded) = {
            let s = app.state::<AppState>();
            let s = s.inner.lock().unwrap();
            (s.placement.clone(), s.placement.embedded)
        };
        let now = taskbar::info();
        let changed = match &now {
            Some((tb, info)) => {
                Some(info.rect) != last.taskbar
                    || (last.floating && taskbar::work_area(*tb) != last.work)
                    || info.centered != last.centered
                    || info.widgets != last.widgets
                    || (info.scale - last.scale).abs() > f64::EPSILON
                    || (embedded && !embed::is_child_of(hwnd, now.as_ref().map(|n| n.0).unwrap_or(0)))
            }
            None => last.found_taskbar,
        };
        if changed {
            apply(app);
        } else if last.visible {
            embed::ensure_top(hwnd, now.filter(|_| embedded).map(|n| n.0));
        }
    }
    #[cfg(not(any(windows, feature = "lint-win")))]
    let _ = app;
}

/// Show the hover card (`w` × `h` logical px) next to the hovered ring, without
/// taking focus. Runs on the UI thread.
pub fn show_tip(app: &AppHandle, w_logical: f64, h_logical: f64) {
    let Some(win) = app.get_webview_window(crate::TIP) else { return };
    let state = app.state::<AppState>();
    let (anchor, placement, position) = {
        let s = state.inner.lock().unwrap();
        (s.tip_anchor, s.placement.clone(), s.config.position)
    };
    let Some(anchor) = anchor else { return };
    let scale = if placement.scale > 0.0 { placement.scale } else { 1.0 };
    let (w, h) = ((w_logical * scale).round() as i32, (h_logical * scale).round() as i32);

    #[cfg(any(windows, feature = "lint-win"))]
    {
        use dwbar_core::{config::Position, layout, win::{embed, taskbar}};
        let Some(hwnd) = hwnd_of(&win) else { return };
        let screen = taskbar::primary()
            .and_then(taskbar::monitor_area)
            .or(placement.taskbar)
            .unwrap_or(layout::Rect::new(0, 0, 1920, 1080));
        let side = match position {
            Position::Left => layout::TipSide::Right,
            Position::Right => layout::TipSide::Left,
            _ => layout::TipSide::Auto,
        };
        let gap = (8.0 * scale).round() as i32;
        let rect = layout::place_tip(&anchor, &screen, side, gap, w, h);
        embed::place(hwnd, rect, None);
        embed::set_visible(hwnd, true);
    }
    #[cfg(not(any(windows, feature = "lint-win")))]
    {
        let _ = (position, placement);
        let _ = win.set_size(tauri::PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(tauri::PhysicalPosition::new(anchor.left, anchor.top - h - 8));
        let _ = win.show();
    }
}

pub fn hide_tip(app: &AppHandle) {
    let Some(win) = app.get_webview_window(crate::TIP) else { return };
    #[cfg(any(windows, feature = "lint-win"))]
    if let Some(hwnd) = hwnd_of(&win) {
        dwbar_core::win::embed::set_visible(hwnd, false);
    }
    #[cfg(not(any(windows, feature = "lint-win")))]
    let _ = win.hide();
}

/// Show the usage panel (`w` × `h` logical px) next to the capsule, focused so it
/// closes itself when the user clicks elsewhere. Runs on the UI thread.
pub fn show_panel(app: &AppHandle, w_logical: f64, h_logical: f64) {
    let Some(win) = app.get_webview_window(crate::PANEL) else { return };
    let state = app.state::<AppState>();
    let (placement, position) = {
        let s = state.inner.lock().unwrap();
        (s.placement.clone(), s.config.position)
    };
    let Some(anchor) = placement.capsule else { return };
    let scale = if placement.scale > 0.0 { placement.scale } else { 1.0 };
    let (w, h) = ((w_logical * scale).round() as i32, (h_logical * scale).round() as i32);

    #[cfg(any(windows, feature = "lint-win"))]
    {
        use dwbar_core::{config::Position, layout, win::{embed, taskbar}};
        let Some(hwnd) = hwnd_of(&win) else { return };
        let screen = taskbar::primary()
            .and_then(taskbar::monitor_area)
            .or(placement.taskbar)
            .unwrap_or(layout::Rect::new(0, 0, 1920, 1080));
        let side = match position {
            Position::Left => layout::TipSide::Right,
            Position::Right => layout::TipSide::Left,
            _ => layout::TipSide::Auto,
        };
        let gap = (10.0 * scale).round() as i32;
        let rect = layout::place_tip(&anchor, &screen, side, gap, w, h);
        embed::place(hwnd, rect, None);
    }
    #[cfg(not(any(windows, feature = "lint-win")))]
    {
        let _ = position;
        let _ = win.set_size(tauri::PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(tauri::PhysicalPosition::new(anchor.left, anchor.top - h - 10));
    }
    let _ = win.show();
    let _ = win.set_focus();
}

pub fn hide_panel(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(crate::PANEL) {
        let _ = win.hide();
    }
}
