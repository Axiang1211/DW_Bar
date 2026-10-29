// Shared helpers for the capsule and settings pages.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export { invoke, listen };

export const $ = (s, el = document) => el.querySelector(s);
export const $$ = (s, el = document) => [...el.querySelectorAll(s)];

/** Names, monograms (shown until / unless the logo loads) and account wording. */
export const AGENTS = {
  claude: { name: "Claude", mono: "C", cli: true, account: "Claude 帳號", site: "claude.ai" },
  codex: { name: "Codex", mono: "X", cli: true, account: "ChatGPT 帳號", site: "chatgpt.com，讀取 Codex 額度" },
  gemini: { name: "Gemini", mono: "G", account: "Google 帳號", button: "連結 Gemini（Google 帳號）", site: "gemini.google.com" },
  cursor: { name: "Cursor", mono: "Cu", account: "Cursor 帳號", site: "cursor.com，讀取本期方案額度" },
  windsurf: { name: "Windsurf", mono: "W", account: "Windsurf 帳號", site: "windsurf.com，讀取每日／每週額度" },
  deepseek: { name: "DeepSeek", mono: "D", account: "DeepSeek 帳號", site: "platform.deepseek.com，讀取 API 餘額（DSH）" },
  antigravity: { name: "Antigravity", mono: "A", local: true, account: "Antigravity", site: "打開 Antigravity 時自動讀取，不需要登入" },
  dw: { name: "DW Bar", mono: "DW" },
};

/** Remaining percent (the prototype shows what is left, not what is used). */
export const left = (w) => (w ? Math.max(0, Math.min(100, 100 - w.used_percent)) : null);

/** ok ≥ 50 % left, warn ≥ 20 %, crit below. */
export function level(remaining) {
  if (remaining == null) return "none";
  if (remaining >= 50) return "ok";
  if (remaining >= 20) return "warn";
  return "crit";
}

export const win = (reading, period) => reading?.windows?.find((w) => w.period === period) || null;
/** Outer half arc: the short window (5 h, else daily). Inner circle: the long one (week, else month). */
export const shortWin = (r) => win(r, "five_hour") || win(r, "day");
export const longWin = (r) => win(r, "week") || win(r, "month");

/** "14:30" today, "週四 09:00" this week, "10/31 09:00" later. */
export function resetText(unix) {
  if (!unix) return "";
  const d = new Date(unix * 1000);
  const now = new Date();
  const hm = d.toLocaleTimeString("zh-TW", { hour: "2-digit", minute: "2-digit", hour12: false });
  const days = (d - now) / 86400000;
  if (d.toDateString() === now.toDateString()) return hm;
  if (days < 6.5) return `週${"日一二三四五六"[d.getDay()]} ${hm}`;
  return `${d.getMonth() + 1}/${d.getDate()} ${hm}`;
}

/** "還有 2 小時 14 分" */
export function untilText(unix) {
  if (!unix) return "";
  let s = Math.max(0, unix - Date.now() / 1000);
  const d = Math.floor(s / 86400); s -= d * 86400;
  const h = Math.floor(s / 3600); s -= h * 3600;
  const m = Math.floor(s / 60);
  if (d) return `${d} 天 ${h} 小時`;
  if (h) return `${h} 小時 ${m} 分`;
  return `${m} 分`;
}

// Upper half arc (5-hour window) and inner circle (weekly), same geometry as the prototype.
const HALF = "M6.5 50 A43.5 43.5 0 0 1 93.5 50";

/**
 * Each AI's own icon, loaded at runtime from its official site (the same icon its
 * browser tab shows). Nothing is bundled with DW Bar; if every source fails the ring
 * keeps its monogram.
 */
const LOGOS = {
  claude: ["https://claude.ai/apple-touch-icon.png", "https://claude.ai/favicon.ico"],
  codex: ["https://chatgpt.com/apple-touch-icon.png", "https://chatgpt.com/favicon.ico"],
  gemini: ["https://gemini.google.com/favicon.ico", "https://www.google.com/s2/favicons?domain=gemini.google.com&sz=64"],
  antigravity: ["https://antigravity.google/favicon.ico", "https://www.google.com/s2/favicons?domain=antigravity.google&sz=64"],
  cursor: ["https://cursor.com/apple-touch-icon.png", "https://cursor.com/favicon.ico"],
  windsurf: ["https://windsurf.com/apple-touch-icon.png", "https://windsurf.com/favicon.ico"],
  deepseek: ["https://www.deepseek.com/favicon.ico", "https://chat.deepseek.com/favicon.svg"],
};
const badLogos = new Set();

// <img> errors don't bubble, but they do pass the document in the capture phase.
document.addEventListener(
  "error",
  (e) => {
    const img = e.target;
    if (!(img instanceof HTMLImageElement) || !img.classList.contains("logo-img")) return;
    badLogos.add(img.getAttribute("src"));
    const next = (img.dataset.alt || "").split("|").filter((u) => u && !badLogos.has(u));
    if (next.length) {
      img.dataset.alt = next.slice(1).join("|");
      img.src = next[0];
    } else {
      img.parentElement?.remove();
    }
  },
  true,
);

// Pixel style: an SVG filter that turns the icon into a coarse dot matrix.
if (!document.getElementById("dw-defs")) {
  const defs = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  defs.id = "dw-defs";
  defs.setAttribute("width", "0");
  defs.setAttribute("height", "0");
  defs.style.position = "absolute";
  defs.innerHTML = `<filter id="dw-px" x="0" y="0">
      <feFlood x="1" y="1" height="1" width="1"/><feComposite width="3" height="3"/>
      <feTile result="grid"/><feComposite in="SourceGraphic" in2="grid" operator="in"/>
      <feMorphology operator="dilate" radius="1"/></filter>`;
  (document.body || document.documentElement).append(defs);
}

/** Logo style for every ring: mono / neon / acrylic / pixel / color. */
export function applyIconStyle(style) {
  document.documentElement.dataset.icons = style || "mono";
}

function logoHTML(agent) {
  const urls = (LOGOS[agent] || []).filter((u) => !badLogos.has(u));
  if (!urls.length) return "";
  return `<span class="logo"><img class="logo-img" alt="" draggable="false" referrerpolicy="no-referrer"
    src="${urls[0]}" data-alt="${urls.slice(1).join("|")}"></span>`;
}

export function ringSVG(agent, reading) {
  return ringOnly(agent, reading) + logoHTML(agent);
}

function ringOnly(agent, reading) {
  const h5 = left(shortWin(reading));
  const wk = left(longWin(reading));
  const off = (v) => (v == null ? 100 : 100 - v);
  const mono = AGENTS[agent]?.mono || agent.slice(0, 1).toUpperCase();
  return `<svg viewBox="0 0 100 100" aria-hidden="true">
    <path class="trk trk-5" d="${HALF}"/>
    <path class="arc arc-5 lv-${level(h5)}" d="${HALF}" pathLength="100" style="stroke-dashoffset:${off(h5)}"/>
    <circle class="trk trk-w" cx="50" cy="50" r="33"/>
    <circle class="arc arc-w" cx="50" cy="50" r="33" pathLength="100" transform="rotate(-90 50 50)" style="stroke-dashoffset:${off(wk)}"/>
    <text class="mono${mono.length > 1 ? " mono-2" : ""}" x="50" y="50" dominant-baseline="central" text-anchor="middle">${mono}</text>
  </svg>`;
}

/** Apply the configured theme: "auto" follows Windows (prefers-color-scheme). */
export function applyTheme(theme) {
  const root = document.documentElement;
  if (theme === "dark" || theme === "light") root.dataset.theme = theme;
  else delete root.dataset.theme;
}

export function tooltip(reading) {
  const name = reading.name;
  if (!reading.ok) return `${name}\n${reading.notice || "沒有資料"}`;
  const lines = [name];
  for (const w of reading.windows) {
    const r = left(w);
    const reset = w.resets_at ? `（${resetText(w.resets_at)} 重置）` : "";
    lines.push(`${w.label}　剩 ${Math.round(r)}%${reset}`);
  }
  if (reading.plan) lines.push(`方案　${reading.plan}`);
  if (reading.balance) lines.push(`餘額　${reading.balance}`);
  return lines.join("\n");
}
