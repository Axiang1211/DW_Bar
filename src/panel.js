import { $, invoke, listen, ringSVG, left, shortWin, longWin, level, applyTheme, applyIconStyle, resetText, untilText, AGENTS } from "./common.js";
import { applyMaterial } from "./material.js";

let state = null;
let focusAgent = null;
let open = false; // only an open panel may resize / show its window

function close() {
  open = false;
  invoke("hide_panel");
}
const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

function card(q) {
  const a = AGENTS[q.agent] || {};
  const s = shortWin(q);
  const l = longWin(q);
  const main = s || l;
  const r = left(main);
  const glow = main ? `radial-gradient(120% 90% at 0% 0%, color-mix(in srgb, var(--${level(r)}) 18%, transparent), transparent 60%)` : "none";
  let body;
  if (q.ok && main) {
    const rows = [];
    if (s && l) rows.push(`<div><dt><i class="sw" style="background:var(--weekly)"></i>${esc(l.label)}</dt><dd>剩 ${Math.round(left(l))}%</dd></div>`);
    for (const w of q.windows.filter((w) => w !== s && w !== l)) rows.push(`<div><dt>${esc(w.label)}</dt><dd>剩 ${Math.round(left(w))}%</dd></div>`);
    if (main.resets_at) rows.push(`<div><dt>重置</dt><dd>${resetText(main.resets_at)}（還有 ${untilText(main.resets_at)}）</dd></div>`);
    rows.push(`<div><dt>餘額</dt><dd>${esc(q.balance || "—")}</dd></div>`);
    body = `<div class="mc-big"><b>${Math.round(r)}%</b><small>${esc(main.label)} 剩餘</small></div><dl class="mc-rows">${rows.join("")}</dl>`;
  } else if (q.ok) {
    body = `<div class="mc-big"><b>${esc(q.balance || "—")}</b><small>餘額</small></div>`;
  } else {
    const act = a.local ? `<div class="mc-act"><button class="btn" data-refresh>重新偵測</button></div>` : `<div class="mc-act"><button class="btn" data-connect="${q.agent}">${q.connected ? "重新登入" : "連結帳號"}</button></div>`;
    body = `<div class="mc-note">${esc(q.notice || "沒有資料")}</div>${act}`;
  }
  return `<article class="mc${q.agent === focusAgent ? " focus" : ""}">
    <div class="mc-glow" style="background:${glow}"></div>
    <div class="mc-grid"><span class="ring">${ringSVG(q.agent, q)}</span>
      <div><div class="mc-top"><span class="mc-name">${esc(q.name)}</span>${q.plan ? `<span class="chip">${esc(q.plan)}</span>` : ""}</div>${body}</div>
    </div></article>`;
}

function render() {
  if (!state) return;
  applyTheme(state.config.theme);
  applyIconStyle(state.config.icon_style);
  applyMaterial(state.config.material, { theme: state.config.theme, blur: true, platform: state.platform, round: state.round_corners });
  const shown = state.quotas.filter((q) => q.installed);
  const newest = Math.max(0, ...state.quotas.filter((q) => q.ok).map((q) => q.updated));
  $("#sub").textContent = `${shown.length} 個 AI${newest ? ` · 上次更新 ${new Date(newest * 1000).toLocaleTimeString("zh-TW", { hour: "2-digit", minute: "2-digit", hour12: false })}` : ""}`;
  $("#refresh").classList.toggle("spin", !!state.refreshing);
  // The AI that was clicked goes first.
  const ordered = [...shown].sort((a, b) => (b.agent === focusAgent) - (a.agent === focusAgent));
  $("#cards").innerHTML = ordered.length
    ? ordered.map(card).join("")
    : `<div class="empty">還沒有連結任何 AI。<br><button class="link" data-goto="agents">連結帳號</button></div>`;
  $("#foot").textContent = `每 ${state.config.refresh_minutes} 分鐘自動更新`;
  if (!open) return;
  requestAnimationFrame(() => {
    const w = $("#wrap");
    invoke("panel_resize", { width: Math.ceil(w.offsetWidth), height: Math.ceil(w.offsetHeight) });
  });
}

$("#brand").innerHTML = ringSVG("dw", { ok: true, windows: [{ period: "five_hour", used_percent: 36 }, { period: "week", used_percent: 28 }] });
$("#refresh").addEventListener("click", () => invoke("refresh_quotas"));
$("#settings").addEventListener("click", () => { close(); invoke("show_settings", { page: "usage" }); });
$("#accounts").addEventListener("click", () => { close(); invoke("show_settings", { page: "agents" }); });
$("#close").addEventListener("click", close);
document.addEventListener("click", (e) => {
  const c = e.target.closest("[data-connect]");
  if (c) { close(); invoke("connect_account", { agent: c.dataset.connect }); }
  if (e.target.closest("[data-refresh]")) invoke("refresh_quotas");
  const g = e.target.closest("[data-goto]");
  if (g) { close(); invoke("show_settings", { page: g.dataset.goto }); }
});
document.addEventListener("keydown", (e) => e.key === "Escape" && close());
// Close when the user clicks anywhere else.
window.addEventListener("blur", () => setTimeout(() => open && !document.hasFocus() && close(), 80));

listen("state", (e) => { state = e.payload; render(); });
listen("panel-close", () => (open = false));
listen("panel-open", async (e) => {
  focusAgent = e.payload || null;
  open = true;
  state = await invoke("get_state");
  const p = $("#panel");
  p.style.animation = "none"; void p.offsetWidth; p.style.animation = "";
  render();
});
invoke("get_state").then((s) => (state = s));
