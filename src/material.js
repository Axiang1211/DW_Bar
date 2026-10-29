// Background materials for the panel, hover card and capsule (外觀 → 背景材質).
// The OS blurs the desktop behind the panel / card for frosted, liquid and translucent
// colours (see Material::backdrop in Rust); this module paints everything else.

export const KINDS = [
  ["default", "預設"],
  ["frosted", "毛玻璃"],
  ["liquid", "液態玻璃"],
  ["color", "純色"],
  ["gradient", "漸層"],
  ["metal", "液態金屬"],
];

export const GRADIENTS = {
  aurora: { name: "極光", stops: ["#1fd1c1", "#5b6cff", "#b45cff"] },
  sunset: { name: "夕陽", stops: ["#ff9a5a", "#ff5e7e", "#b84cff"] },
  ocean: { name: "深海", stops: ["#0b2a6f", "#1565c0", "#26c6da"] },
  sakura: { name: "櫻花", stops: ["#ffd1e3", "#ffb3cf", "#c8b6ff"] },
  forest: { name: "森林", stops: ["#0f3d2e", "#1b7a50", "#8bd17c"] },
  dream: { name: "夢翔", stops: ["#3d8bff", "#8fc6ff", "#ffd36b"] },
  night: { name: "夜空", stops: ["#0f1021", "#2b2d64", "#5a3d8a"] },
  candy: { name: "糖果", stops: ["#ffe0ef", "#e2d4ff", "#c7f0ff"] },
  lava: { name: "熔岩", stops: ["#2b0a0a", "#a3201c", "#ff8a3d"] },
};

// Liquid metal: tight, alternating highlight / shadow bands, like polished chrome.
export const METALS = {
  silver: { name: "液態銀", ink: "dark", stops: ["#eef1f5 0%", "#a7adb7 16%", "#fbfcfd 30%", "#8c939e 46%", "#dfe3e8 62%", "#9aa1ab 78%", "#f4f6f8 100%"] },
  gold: { name: "液態金", ink: "dark", stops: ["#fff3c4 0%", "#c9a13b 16%", "#fff6d8 30%", "#a8822a 46%", "#f3d98a 62%", "#b89230 78%", "#fff0bd 100%"] },
  rose: { name: "玫瑰金", ink: "dark", stops: ["#ffe4dc 0%", "#d1958a 16%", "#fff0eb 30%", "#b97a6f 46%", "#f4c3b6 62%", "#c4887d 78%", "#ffe9e2 100%"] },
  titanium: { name: "鈦黑", ink: "light", stops: ["#5d636c 0%", "#2a2e34 16%", "#7a818b 30%", "#1f2227 46%", "#565c65 62%", "#25292e 78%", "#6e757e 100%"] },
  holo: { name: "幻彩", ink: "dark", stops: ["#ffd6f5 0%", "#c9e7ff 20%", "#e9ffd9 38%", "#fff1c9 55%", "#e0d0ff 72%", "#c8fff4 88%", "#ffd6f5 100%"] },
};

const hexRgb = (hex) => {
  const h = String(hex || "").trim().replace("#", "");
  const v = parseInt(h.length === 3 ? h.replace(/./g, "$&$&") : h, 16);
  return Number.isFinite(v) && h.length >= 3 ? [(v >> 16) & 255, (v >> 8) & 255, v & 255] : [61, 139, 255];
};
const rgba = (hex, a) => `rgba(${hexRgb(hex).join(",")},${a})`;
const luminance = (hex) => {
  const [r, g, b] = hexRgb(hex).map((c) => {
    c /= 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};

function gradientStops(m) {
  if (m.gradient === "custom") return { stops: m.colors, angle: m.angle };
  return { stops: (GRADIENTS[m.gradient] || GRADIENTS.aurora).stops, angle: 135 };
}

/** Is the OS theme (or the chosen theme) dark? */
export function prefersDark(theme) {
  if (theme === "dark") return true;
  if (theme === "light") return false;
  return !matchMedia("(prefers-color-scheme: light)").matches;
}

/**
 * CSS background and text ink ("light" text on dark, "dark" text on light) for a material.
 * Returns null for materials drawn purely by CSS (default, frosted, liquid).
 */
export function paint(m, dark) {
  const a = Math.min(100, Math.max(20, m.opacity ?? 85)) / 100;
  // What shows through a translucent colour: assume the theme's backdrop.
  const behind = dark ? 0.03 : 0.85;
  const ink = (lums) => {
    const avg = lums.reduce((s, l) => s + l, 0) / lums.length;
    return avg * a + behind * (1 - a) > 0.36 ? "dark" : "light";
  };
  if (m.kind === "color") {
    return { bg: rgba(m.color, a), ink: ink([luminance(m.color)]) };
  }
  if (m.kind === "gradient") {
    const { stops, angle } = gradientStops(m);
    return { bg: `linear-gradient(${angle}deg, ${stops.map((c) => rgba(c, a)).join(", ")})`, ink: ink(stops.map(luminance)) };
  }
  if (m.kind === "metal") {
    const metal = METALS[m.metal] || METALS.silver;
    const sheen = "linear-gradient(115deg, transparent 32%, rgba(255,255,255,0.55) 46%, rgba(255,255,255,0.08) 54%, transparent 64%)";
    return { bg: `${sheen} 0 0 / 260% 100%, linear-gradient(135deg, ${metal.stops.join(", ")})`, ink: metal.ink };
  }
  return null;
}

/** True when the OS blurs the desktop behind this material (mirrors Material::backdrop). */
export function osBlurs(m) {
  const k = m?.kind;
  return k === "frosted" || ((k === "color" || k === "gradient") && (m.opacity ?? 85) < 95);
}

/**
 * Apply a material to this page (or to `target`, for previews). `blur`: the OS is blurring behind this window (panel and
 * hover card only), so the page drops its shadow margin and matches the OS corner radius.
 */
export function applyMaterial(m, { theme = "auto", blur = false, platform = "windows", round = true, target = document.documentElement } = {}) {
  const root = target;
  const kind = m?.kind || "default";
  const dark = prefersDark(theme);
  if (kind === "default") {
    delete root.dataset.material;
    delete root.dataset.ink;
    delete root.dataset.blur;
    root.style.removeProperty("--m-bg");
    return;
  }
  root.dataset.material = kind;
  const p = paint(m, dark);
  if (p) root.style.setProperty("--m-bg", p.bg);
  else root.style.removeProperty("--m-bg");
  // Clear liquid glass can't know what is behind it: white text with a shadow reads
  // over the most backgrounds.
  root.dataset.ink = p ? p.ink : kind === "liquid" || dark ? "light" : "dark";
  // Glass tint strength, 0..0.8 (frosted / liquid).
  root.style.setProperty("--m-tint", String(Math.min(80, Math.max(0, m.tint ?? 20)) / 100));
  if (blur && osBlurs(m)) {
    root.dataset.blur = "1";
    root.style.setProperty("--blur-r", platform === "macos" ? "12px" : round ? "8px" : "0px");
  } else {
    delete root.dataset.blur;
  }
}
