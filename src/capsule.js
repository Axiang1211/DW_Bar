import { $, invoke, listen, ringSVG, left, shortWin, longWin, level, tooltip, applyTheme } from "./common.js";

const cap = $("#cap");
let lastWidth = 0;

function render(state) {
  applyTheme(state.config.theme);
  const shown = state.quotas.filter((q) => q.installed);
  cap.replaceChildren();

  if (!shown.length) {
    // Nothing detected yet: one brand ring that opens AI detection settings.
    const b = item("dw", null, state.refreshing && !state.quotas.length ? "正在偵測 AI…" : "DW Bar：點一下連結你的 AI 帳號", "agents");
    cap.append(b);
  } else {
    for (const q of shown) {
      const el = item(q.agent, q, tooltip(q), q.ok ? "usage" : "agents");
      if (!q.ok) el.classList.add("off");
      if (q.ok && !q.windows.length && q.balance) {
        // Balance-only AIs (DeepSeek API) have no rings to fill; show the balance itself.
        const p = document.createElement("span");
        p.className = "pct";
        p.textContent = q.balance;
        el.append(p);
      } else if (state.config.show_percent && q.ok) {
        const r = left(shortWin(q)) ?? left(longWin(q));
        if (r != null) {
          const p = document.createElement("span");
          p.className = `pct lv-${level(r)}`;
          p.innerHTML = `${Math.round(r)}<small>%</small>`;
          el.append(p);
        }
      }
      cap.append(el);
    }
  }
  cap.classList.toggle("spin", !!state.refreshing && !state.quotas.length);
  requestAnimationFrame(reportWidth);
}

function item(agent, reading, title, page) {
  const b = document.createElement("button");
  b.className = "item";
  b.title = title;
  b.setAttribute("aria-label", title.split("\n").join("，"));
  const ring = document.createElement("span");
  ring.className = "ring";
  ring.innerHTML = ringSVG(agent, reading);
  b.append(ring);
  b.addEventListener("click", () => invoke("show_settings", { page }));
  return b;
}

/** Tell Rust how wide the capsule wants to be (logical px); it sizes and places the window. */
function reportWidth() {
  const w = Math.ceil(cap.getBoundingClientRect().width);
  if (w && Math.abs(w - lastWidth) >= 1) {
    lastWidth = w;
    invoke("capsule_resize", { width: w }).catch(() => {});
  }
}

new ResizeObserver(reportWidth).observe(document.body);
listen("state", (e) => render(e.payload));
invoke("get_state").then(render);
// Reset times in tooltips go stale; re-render once a minute.
setInterval(() => invoke("get_state").then(render), 60_000);
