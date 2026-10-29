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
        // macOS (and other systems): no Windows taskbar, so the capsule is a floating HUD.
        // The "taskbar" cell of the grid means just above the Dock.
        use crate::state::Placement;
        use dwbar_core::{config::Position, layout};
        let (config, width_logical, height_logical) = {
            let s = state.inner.lock().unwrap();
            (s.config.clone(), s.capsule_width, s.capsule_height)
        };
        let Some((screen, work, scale)) = monitor_areas(&win) else { return };
        let pos = if config.position == Position::Taskbar { Position::Bottom } else { config.position };
        let thick = (layout::HUD_THICKNESS * scale).round() as i32;
        let (w, h) = if pos.vertical() {
            (thick, ((height_logical * scale).round() as i32).max(thick))
        } else {
            (((width_logical * scale).round() as i32).max(thick), thick)
        };
        let rect = layout::place_floating(&work, pos, scale, w, h);
        let _ = win.set_size(tauri::PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(tauri::PhysicalPosition::new(rect.left, rect.top));
        let _ = win.show();
        let mut s = state.inner.lock().unwrap();
        s.placement = Placement {
            found_taskbar: false,
            embedded: false,
            fallback: false,
            visible: true,
            scale,
            centered: false,
            widgets: false,
            taskbar: Some(screen),
            capsule: Some(rect),
            work: Some(work),
            floating: true,
        };
    }
}

/// Whole screen, work area (without menu bar / Dock / taskbar) and scale of the window's monitor.
#[cfg_attr(any(windows, feature = "lint-win"), allow(dead_code))]
fn monitor_areas(win: &WebviewWindow) -> Option<(dwbar_core::layout::Rect, dwbar_core::layout::Rect, f64)> {
    use dwbar_core::layout::Rect;
    let mon = win.current_monitor().ok().flatten().or_else(|| win.primary_monitor().ok().flatten())?;
    let (p, z) = (mon.position(), mon.size());
    let screen = Rect::new(p.x, p.y, p.x + z.width as i32, p.y + z.height as i32);
    let wa = mon.work_area();
    let work = Rect::new(
        wa.position.x,
        wa.position.y,
        wa.position.x + wa.size.width as i32,
        wa.position.y + wa.size.height as i32,
    );
    Some((screen, if work.is_empty() { screen } else { work }, mon.scale_factor()))
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
        use dwbar_core::{config::Position, layout};
        let screen = monitor_areas(&win).map(|m| m.0).or(placement.taskbar).unwrap_or(layout::Rect::new(0, 0, 1920, 1080));
        let side = match position {
            Position::Left => layout::TipSide::Right,
            Position::Right => layout::TipSide::Left,
            _ => layout::TipSide::Auto,
        };
        let rect = layout::place_tip(&anchor, &screen, side, (8.0 * scale).round() as i32, w, h);
        let _ = win.set_size(tauri::PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(tauri::PhysicalPosition::new(rect.left, rect.top));
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
        use dwbar_core::{config::Position, layout};
        let screen = monitor_areas(&win).map(|m| m.0).or(placement.taskbar).unwrap_or(layout::Rect::new(0, 0, 1920, 1080));
        let side = match position {
            Position::Left => layout::TipSide::Right,
            Position::Right => layout::TipSide::Left,
            _ => layout::TipSide::Auto,
        };
        let rect = layout::place_tip(&anchor, &screen, side, (10.0 * scale).round() as i32, w, h);
        let _ = win.set_size(tauri::PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(tauri::PhysicalPosition::new(rect.left, rect.top));
    }
    let _ = win.show();
    let _ = win.set_focus();
}

pub fn hide_panel(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(crate::PANEL) {
        let _ = win.hide();
    }
}

/// Blur the desktop behind the panel / hover card for the chosen material.
/// Windows: accent blur on DW Bar's own window; macOS: vibrancy. Run on the UI thread.
pub fn set_backdrop(win: &WebviewWindow, backdrop: dwbar_core::config::Backdrop) {
    #[cfg(any(windows, feature = "lint-win"))]
    if let Some(hwnd) = hwnd_of(win) {
        dwbar_core::win::backdrop::set(hwnd, backdrop);
    }
    #[cfg(target_os = "macos")]
    {
        use dwbar_core::config::Backdrop;
        use tauri::window::{Effect, EffectState, EffectsBuilder};
        let effect = match backdrop {
            Backdrop::None => None,
            Backdrop::Blur => Some(Effect::UnderWindowBackground),
            Backdrop::Acrylic => Some(Effect::HudWindow),
        };
        let config = effect.map(|e| EffectsBuilder::new().effect(e).state(EffectState::Active).radius(12.0).build());
        if let Err(e) = win.set_effects(config) {
            dwbar_core::log::write("backdrop", &format!("vibrancy: {e}"));
        }
    }
    #[cfg(not(any(windows, feature = "lint-win", target_os = "macos")))]
    let _ = (win, backdrop);
}
