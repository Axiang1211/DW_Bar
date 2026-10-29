import { $, invoke, listen, left, level, applyTheme, resetText, untilText, t, tb, setLang } from "./common.js";
import { applyMaterial } from "./material.js";

let state = null;
let agent = null;
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

function render() {
  if (!state || !agent) return;
  setLang(state.config.language);
  applyTheme(state.config.theme);
  applyMaterial(state.config.material, { theme: state.config.theme, blur: true, platform: state.platform, round: state.round_corners });
  const q = state.quotas.find((x) => x.agent === agent);
  if (!q) return;
  const pos = state.config.position || "taskbar";
  const corner = ["tl", "tr", "bl", "br"].includes(pos);
  // Same layout as the prototype's hover card: name + plan, one row per window, balance, hint.
  const head = `<div class="tip-head"><b>${esc(q.name)}</b><small>${esc(q.plan || q.account || "")}</small></div>`;
  let rows = "";
  if (q.ok) {
    rows = q.windows
      .map((w) => {
        const r = left(w);
        const long = w.period === "week" || w.period === "month";
        const when = w.resets_at ? t("{left}後重置", { left: untilText(w.resets_at) }) : "";
        const color = long ? "var(--weekly)" : `var(--${level(r)})`;
        return `<div class="tip-row"><span>${esc(tb(w.label))}</span><em>${t("剩 {n}%", { n: Math.round(r) })}<small>${when}</small></em>
          <span class="tip-bar"><i style="width:${r}%;background:${color}"></i></span></div>`;
      })
      .join("");
  } else {
    rows = `<div class="tip-row"><span>${esc(tb(q.notice) || t("沒有資料"))}</span></div>`;
  }
  rows += `<div class="tip-row"><span>${t("餘額")}</span><em>${esc(tb(q.balance) || "—")}</em></div>`;
  const next = q.windows?.filter((w) => w.resets_at).sort((a, b) => a.resets_at - b.resets_at)[0];
  const foot = [next ? t("下次重置 {time}", { time: resetText(next.resets_at) }) : "", corner ? t("點擊展開全部 AI") : t("點擊開啟面板")]
    .filter(Boolean)
    .join(" · ");
  $("#card").innerHTML = `${head}<div class="tip-rows">${rows}</div><div class="tip-foot">${foot}</div>`;
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
