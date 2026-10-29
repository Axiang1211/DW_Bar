//! Where the capsule goes. Pure geometry so it can be unit-tested anywhere.
//!
//! Rules (spec §2, "工作列嵌入與定位規則"):
//! * `Auto` follows the Windows widgets rule: centred taskbar (TaskbarAl = 1) → far left;
//!   left-aligned taskbar → immediately left of the system tray.
//! * When the widgets button is on and the taskbar is centred, the far-left slot starts
//!   after the space widgets uses, so the two never overlap.
//! * `offset_x` (logical px) nudges the result; the capsule is always kept on the taskbar.
use crate::config::{Position, Slot};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    pub fn is_empty(&self) -> bool {
        self.width() <= 0 || self.height() <= 0
    }
    /// The same rect relative to `origin`'s top-left (screen → client of a parent).
    pub fn relative_to(&self, origin: &Rect) -> Rect {
        Rect::new(
            self.left - origin.left,
            self.top - origin.top,
            self.right - origin.left,
            self.bottom - origin.top,
        )
    }
    /// Whether `self` covers all of `other` (fullscreen test).
    pub fn covers(&self, other: &Rect) -> bool {
        self.left <= other.left
            && self.top <= other.top
            && self.right >= other.right
            && self.bottom >= other.bottom
    }
}

/// Everything known about the taskbar at the moment of placement (physical px).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct TaskbarInfo {
    pub rect: Rect,
    /// `TrayNotifyWnd` – the clock / tray area, when found.
    pub tray: Option<Rect>,
    /// `TaskbarAl == 1` (Windows 11 default).
    pub centered: bool,
    /// `TaskbarDa == 1`: the widgets button is shown.
    pub widgets: bool,
    /// DPI scale of the taskbar's monitor (1.0 = 96 dpi).
    pub scale: f64,
}

impl TaskbarInfo {
    pub fn vertical(&self) -> bool {
        self.rect.height() > self.rect.width()
    }
}

/// Logical px left free for the widgets button on a centred taskbar.
pub const WIDGETS_RESERVE: f64 = 156.0;
/// Logical px between the capsule and its neighbour.
pub const GAP: f64 = 8.0;
/// Logical px to keep clear of the right edge when the tray is not found.
const TRAY_FALLBACK: f64 = 320.0;

/// The capsule's height for a taskbar: taskbar height minus a 4 px margin top and bottom.
pub fn capsule_height(tb: &TaskbarInfo) -> i32 {
    let thickness = if tb.vertical() { tb.rect.width() } else { tb.rect.height() };
    (thickness - (8.0 * tb.scale).round() as i32).max(16)
}

/// Screen rect for a capsule of `width` × `height` physical px.
pub fn place(tb: &TaskbarInfo, slot: Slot, offset_x: i32, width: i32, height: i32) -> Rect {
    let px = |logical: f64| (logical * tb.scale).round() as i32;
    let r = tb.rect;

    if tb.vertical() {
        // Vertical taskbars (third-party tools): stack under the top edge.
        let x = r.left + (r.width() - width) / 2;
        let y = r.top + px(GAP) + px(offset_x as f64);
        let y = y.clamp(r.top, (r.bottom - height).max(r.top));
        return Rect::new(x, y, x + width, y + height);
    }

    let far_left = || {
        let reserve = if tb.widgets && tb.centered { px(WIDGETS_RESERVE) } else { 0 };
        r.left + px(GAP) + reserve
    };
    let by_tray = || match tb.tray {
        Some(tray) if !tray.is_empty() => tray.left - px(GAP) - width,
        _ => r.right - px(TRAY_FALLBACK) - width,
    };

    let x = match slot {
        Slot::Left => far_left(),
        Slot::Tray => by_tray(),
        Slot::Auto if tb.centered => far_left(),
        Slot::Auto => by_tray(),
    } + px(offset_x as f64);

    let x = x.clamp(r.left, (r.right - width).max(r.left));
    let y = r.top + (r.height() - height) / 2;
    Rect::new(x, y, x + width, y + height)
}

/// Thickness of the floating HUD in logical px (its height, or width when vertical).
pub const HUD_THICKNESS: f64 = 44.0;
/// Logical px between the floating HUD and the screen edge.
pub const HUD_MARGIN: f64 = 12.0;

/// Screen rect for the floating HUD inside the monitor's work area (which already
/// excludes the taskbar). `w` × `h` are physical px.
pub fn place_floating(work: &Rect, pos: Position, scale: f64, w: i32, h: i32) -> Rect {
    let m = (HUD_MARGIN * scale).round() as i32;
    let cx = work.left + (work.width() - w) / 2;
    let cy = work.top + (work.height() - h) / 2;
    let (left, right) = (work.left + m, work.right - m - w);
    let (top, bottom) = (work.top + m, work.bottom - m - h);
    let (x, y) = match pos {
        Position::TopLeft => (left, top),
        Position::Top => (cx, top),
        Position::TopRight => (right, top),
        Position::Left => (left, cy),
        Position::Right => (right, cy),
        Position::BottomLeft => (left, bottom),
        Position::Bottom => (cx, bottom),
        Position::BottomRight => (right, bottom),
        Position::Taskbar => (cx, bottom),
    };
    Rect::new(x, y, x + w, y + h)
}

/// Which side of the hovered ring the usage card opens on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipSide {
    /// Above when the ring is in the lower half of the screen, else below.
    Auto,
    /// To the right of the ring (HUD on the left edge).
    Right,
    /// To the left of the ring (HUD on the right edge).
    Left,
}

/// Screen rect for the hover card (`w` × `h`, physical px) next to `anchor`, kept on `screen`.
pub fn place_tip(anchor: &Rect, screen: &Rect, side: TipSide, gap: i32, w: i32, h: i32) -> Rect {
    let clamp_x = |x: i32| x.clamp(screen.left + gap, (screen.right - gap - w).max(screen.left + gap));
    let clamp_y = |y: i32| y.clamp(screen.top + gap, (screen.bottom - gap - h).max(screen.top + gap));
    let (x, y) = match side {
        TipSide::Right => (clamp_x(anchor.right + gap), clamp_y(anchor.top + (anchor.height() - h) / 2)),
        TipSide::Left => (clamp_x(anchor.left - gap - w), clamp_y(anchor.top + (anchor.height() - h) / 2)),
        TipSide::Auto => {
            let x = clamp_x(anchor.left + (anchor.width() - w) / 2);
            let above = anchor.top - gap - h;
            let below = anchor.bottom + gap;
            let lower_half = anchor.top + anchor.height() / 2 > screen.top + screen.height() / 2;
            let y = if lower_half {
                if above >= screen.top { above } else { below }
            } else if below + h <= screen.bottom {
                below
            } else {
                above
            };
            (x, y)
        }
    };
    Rect::new(x, y, x + w, y + h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(centered: bool, widgets: bool, scale: f64) -> TaskbarInfo {
        let h = (48.0 * scale) as i32;
        TaskbarInfo {
            rect: Rect::new(0, 1080 - h, 1920, 1080),
            tray: Some(Rect::new(1600, 1080 - h, 1920, 1080)),
            centered,
            widgets,
            scale,
        }
    }

    #[test]
    fn auto_on_centered_taskbar_is_far_left() {
        let r = place(&bar(true, false, 1.0), Slot::Auto, 0, 120, 40);
        assert_eq!(r, Rect::new(8, 1036, 128, 1076));
    }

    #[test]
    fn auto_on_left_aligned_taskbar_sits_by_the_tray() {
        let r = place(&bar(false, false, 1.0), Slot::Auto, 0, 120, 40);
        assert_eq!(r.right, 1600 - 8);
    }

    #[test]
    fn widgets_space_is_kept_free_when_centered() {
        let r = place(&bar(true, true, 1.0), Slot::Auto, 0, 120, 40);
        assert_eq!(r.left, 8 + 156);
        // …but not when the taskbar is left-aligned (widgets then live by the tray side).
        let l = place(&bar(false, true, 1.0), Slot::Left, 0, 120, 40);
        assert_eq!(l.left, 8);
    }

    #[test]
    fn scale_and_offset_apply() {
        let r = place(&bar(true, false, 1.5), Slot::Left, 10, 180, 60);
        assert_eq!(r.left, 12 + 15);
        assert_eq!(capsule_height(&bar(true, false, 1.5)), 72 - 12);
    }

    #[test]
    fn never_leaves_the_taskbar() {
        let r = place(&bar(true, false, 1.0), Slot::Left, -500, 120, 40);
        assert_eq!(r.left, 0);
        let r = place(&bar(true, false, 1.0), Slot::Tray, 5000, 120, 40);
        assert_eq!(r.right, 1920);
    }

    #[test]
    fn missing_tray_uses_fallback() {
        let mut tb = bar(false, false, 1.0);
        tb.tray = None;
        let r = place(&tb, Slot::Tray, 0, 100, 40);
        assert_eq!(r.right, 1920 - 320);
    }

    #[test]
    fn floating_positions() {
        let work = Rect::new(0, 0, 1920, 1032);
        assert_eq!(place_floating(&work, Position::TopLeft, 1.0, 100, 44), Rect::new(12, 12, 112, 56));
        assert_eq!(place_floating(&work, Position::BottomRight, 1.0, 100, 44), Rect::new(1808, 976, 1908, 1020));
        let r = place_floating(&work, Position::Left, 1.5, 66, 300);
        assert_eq!((r.left, r.top), (18, 366));
        let t = place_floating(&work, Position::Top, 1.0, 200, 44);
        assert_eq!(t.left, 860);
    }

    #[test]
    fn tip_placement() {
        let screen = Rect::new(0, 0, 1920, 1080);
        // ring on the taskbar → card above it, centred, kept on screen
        let r = place_tip(&Rect::new(1500, 1040, 1532, 1072), &screen, TipSide::Auto, 8, 260, 150);
        assert_eq!(r, Rect::new(1386, 882, 1646, 1032));
        // ring at the top → below
        let r = place_tip(&Rect::new(10, 12, 42, 44), &screen, TipSide::Auto, 8, 260, 150);
        assert_eq!((r.left, r.top), (8, 52));
        // left-edge HUD → to the right
        let r = place_tip(&Rect::new(12, 500, 56, 544), &screen, TipSide::Right, 8, 260, 150);
        assert_eq!((r.left, r.top), (64, 447));
    }

    #[test]
    fn relative_and_covers() {
        let parent = Rect::new(0, 1032, 1920, 1080);
        let child = Rect::new(8, 1036, 128, 1076);
        assert_eq!(child.relative_to(&parent), Rect::new(8, 4, 128, 44));
        assert!(Rect::new(0, 0, 1920, 1080).covers(&Rect::new(0, 0, 1920, 1080)));
        assert!(!Rect::new(0, 0, 1920, 1040).covers(&Rect::new(0, 0, 1920, 1080)));
    }
}
