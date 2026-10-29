import { $, invoke, listen, ringSVG, left, level, applyTheme, resetText, untilText } from "./common.js";

let state = null;
let agent = null;
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

function render() {
  if (!state || !agent) return;
  applyTheme(state.config.theme);
  const q = state.quotas.find((x) => x.agent === agent);
  if (!q) return;
  const sub = [q.plan, q.account].filter(Boolean).join(" · ");
  let body = "";
  if (q.ok) {
    body = q.windows
      .map((w) => {
        const r = left(w);
        const long = w.period === "week" || w.period === "month";
        const reset = w.resets_at ? `${resetText(w.resets_at)} 重置 · 還有 ${untilText(w.resets_at)}` : "";
        return `<div class="row"><div class="top"><span>${esc(w.label)}</span><b>剩 ${Math.round(r)}%</b></div>
          <div class="track"><div class="fill ${long ? "wk" : "lv-" + level(r)}" style="width:${r}%"></div></div>
          <small>${reset}</small></div>`;
      })
      .join("");
  } else {
    body = `<div class="notice">${esc(q.notice || "沒有資料")}</div>`;
  }
  $("#card").innerHTML = `<div class="head"><span class="ring">${ringSVG(q.agent, q)}</span>
      <div><b>${esc(q.name)}</b><span>${esc(sub)}</span></div></div>
    ${body}
    <div class="bal"><span>餘額</span><b>${esc(q.balance || "—")}</b></div>`;
  requestAnimationFrame(() => {
    const wrap = $("#wrap");
    invoke("tip_resize", { width: Math.ceil(wrap.offsetWidth), height: Math.ceil(wrap.offsetHeight) });
  });
}

listen("state", (e) => {
  state = e.payload;
  render();
});
listen("tip", async (e) => {
  agent = e.payload;
  if (!state) state = await invoke("get_state");
  // Replay the entry animation for each new ring.
  const card = $("#card");
  card.style.animation = "none";
  void card.offsetWidth;
  card.style.animation = "";
  render();
});
invoke("get_state").then((s) => (state = s));
