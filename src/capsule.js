import { $, invoke, listen, ringSVG, left, shortWin, longWin, level, tooltip, applyTheme } from "./common.js";

const cap = $("#cap");
let lastSize = "";
let last = null;

// Corners show one ring until clicked, then the whole row; back to one after
// 2 minutes without the mouse on it (as in the prototype).
const CORNERS = ["tl", "tr", "bl", "br"];
let expanded = false;
let collapseTimer = 0;
function armCollapse() {
  clearTimeout(collapseTimer);
  collapseTimer = setTimeout(() => {
    expanded = false;
    if (last) render(last);
  }, 120_000);
}
cap.addEventListener("pointermove", () => expanded && armCollapse());

/** The AI that most needs attention: lowest remaining short-window quota. */
function mostUrgent(list) {
  const score = (q) => (q.ok ? left(shortWin(q)) ?? left(longWin(q)) ?? 101 : 102);
  return [...list].sort((a, b) => score(a) - score(b))[0];
}

function render(state) {
  last = state;
  applyTheme(state.config.theme);
  const pos = state.config.position || "taskbar";
  const vertical = pos === "l" || pos === "r";
  const corner = CORNERS.includes(pos);
  document.body.className = [
    pos === "taskbar" ? "on-taskbar" : "floating",
    vertical ? "vertical" : "",
    `pos-${pos}`,
  ].join(" ");

  let shown = state.quotas.filter((q) => q.installed);
  const collapsed = corner && !expanded && shown.length > 1;
  if (collapsed) shown = [mostUrgent(shown)];
  cap.replaceChildren();

  if (!shown.length) {
    // Nothing detected yet: one brand ring that opens account linking.
    const b = item("dw", null, state.refreshing && !state.quotas.length ? "正在偵測 AI…" : "DW Bar：點一下連結你的 AI 帳號");
    b.addEventListener("click", () => invoke("show_settings", { page: "agents" }));
    cap.append(b);
  } else {
    shown.forEach((q, i) => {
      const el = item(q.agent, q, collapsed ? `${tooltip(q)}\n（點一下展開全部）` : tooltip(q));
      if (!q.ok) el.classList.add("off");
      if (!collapsed && corner) el.style.animationDelay = `${i * 40}ms`;
      if (q.ok && !q.windows.length && q.balance) {
        // Balance-only AIs (DeepSeek API) have no rings to fill; show the balance itself.
        if (!vertical) el.append(text("pct", q.balance));
      } else if (state.config.show_percent && q.ok && !vertical) {
        const r = left(shortWin(q)) ?? left(longWin(q));
        if (r != null) {
          const p = text(`pct lv-${level(r)}`, "");
          p.innerHTML = `${Math.round(r)}<small>%</small>`;
          el.append(p);
        }
      }
      el.addEventListener("click", () => {
        invoke("hide_tip").catch(() => {});
        if (collapsed) {
          expanded = true;
          armCollapse();
          render(state);
        } else {
          invoke("show_settings", { page: q.ok ? "usage" : "agents" });
        }
      });
      cap.append(el);
    });
    cap.classList.toggle("expanding", corner && !collapsed);
  }
  cap.classList.toggle("spin", !!state.refreshing && !state.quotas.length);
  requestAnimationFrame(reportSize);
}

function text(cls, value) {
  const s = document.createElement("span");
  s.className = cls;
  s.textContent = value;
  return s;
}

// Hover card: a separate DW Bar window shows the usage of the ring under the pointer.
let tipTimer = 0;
function hoverOn(el, agent) {
  el.addEventListener("pointerenter", () => {
    clearTimeout(tipTimer);
    const r = el.querySelector(".ring").getBoundingClientRect();
    invoke("show_tip", { agent, left: r.left, top: r.top, width: r.width, height: r.height }).catch(() => {});
  });
  el.addEventListener("pointerleave", () => {
    clearTimeout(tipTimer);
    tipTimer = setTimeout(() => invoke("hide_tip").catch(() => {}), 120);
  });
}
window.addEventListener("blur", () => invoke("hide_tip").catch(() => {}));

function item(agent, reading, title) {
  const b = document.createElement("button");
  b.className = "item";
  b.setAttribute("aria-label", title.split("\n").join("，"));
  if (agent !== "dw") hoverOn(b, agent);
  const ring = document.createElement("span");
  ring.className = "ring";
  ring.innerHTML = ringSVG(agent, reading);
  b.append(ring);
  return b;
}

/** Tell Rust how big the capsule wants to be (logical px); it sizes and places the window. */
function reportSize() {
  const r = cap.getBoundingClientRect();
  const w = Math.ceil(r.width);
  const h = Math.ceil(r.height);
  const key = `${w}x${h}`;
  if (w && h && key !== lastSize) {
    lastSize = key;
    invoke("capsule_resize", { width: w, height: h }).catch(() => {});
  }
}

new ResizeObserver(reportSize).observe(cap);
listen("state", (e) => render(e.payload));
invoke("get_state").then(render);
// Reset times in tooltips go stale; re-render once a minute.
setInterval(() => invoke("get_state").then(render), 60_000);
