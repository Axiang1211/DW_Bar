// Shared helpers for the capsule and settings pages.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export { invoke, listen };

export const $ = (s, el = document) => el.querySelector(s);
export const $$ = (s, el = document) => [...el.querySelectorAll(s)];

/** Short names and monograms. Original letterforms – no vendor logos are drawn. */
export const AGENTS = {
  claude: { name: "Claude Code", mono: "C" },
  codex: { name: "Codex", mono: "X" },
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

export function ringSVG(agent, reading) {
  const h5 = left(win(reading, "five_hour"));
  const wk = left(win(reading, "week"));
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
