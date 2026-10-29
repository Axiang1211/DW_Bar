import {
  $, $$, invoke, listen, ringSVG, left, win, level, applyTheme, resetText, untilText, AGENTS,
} from "./common.js";

let state = null;
let config = null;

// ───────── navigation ─────────
function goto(page) {
  if (!document.getElementById("page-" + page)) page = "usage";
  $$(".page").forEach((p) => p.classList.toggle("on", p.id === "page-" + page));
  $("main").scrollTop = 0;
  $$(".side a").forEach((a) =>
    a.dataset.page === page ? a.setAttribute("aria-current", "page") : a.removeAttribute("aria-current"),
  );
  if (page === "diag") loadLog();
}
window.addEventListener("hashchange", () => goto(location.hash.slice(1)));
listen("goto", (e) => { location.hash = e.payload; });
listen("account-linked", (e) => {
  toast(`已連結 ${AGENTS[e.payload]?.account || "帳號"}`);
  location.hash = "usage";
});

// ───────── saving ─────────
let saveTimer = 0;
function save(patch) {
  Object.assign(config, patch);
  applyTheme(config.theme);
  clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    saveTimer = 0;
    try {
      state = await invoke("save_config", { config });
      toast("已儲存");
      renderAll();
    } catch (err) {
      toast(String(err));
    }
  }, 250);
}

function toast(text) {
  const t = $("#toast");
  t.textContent = text;
  t.classList.add("on");
  clearTimeout(toast.t);
  toast.t = setTimeout(() => t.classList.remove("on"), 1600);
}

// ───────── controls bound to config ─────────
function bindControls() {
  $$(".seg").forEach((seg) => {
    seg.addEventListener("click", (e) => {
      const b = e.target.closest("button");
      if (b) save({ [seg.dataset.key]: b.value });
    });
  });
  $$('input[type="checkbox"][data-key]').forEach((cb) => {
    cb.addEventListener("change", () => save({ [cb.dataset.key]: cb.checked }));
  });
  const offset = $("#offset");
  offset.addEventListener("change", () => save({ offset_x: parseInt(offset.value || "0", 10) }));
  $$(".num button").forEach((b) =>
    b.addEventListener("click", () => {
      offset.value = (parseInt(offset.value || "0", 10) + parseInt(b.dataset.step, 10)).toString();
      save({ offset_x: parseInt(offset.value, 10) });
    }),
  );
  $("#refreshMin").addEventListener("change", (e) => save({ refresh_minutes: parseInt(e.target.value, 10) }));
  $("#refresh").addEventListener("click", () => {
    invoke("refresh_quotas");
    $("#refresh").disabled = true;
  });
  $("#quit").addEventListener("click", () => invoke("quit"));
  document.addEventListener("click", (e) => {
    const c = e.target.closest("[data-connect]");
    if (c) {
      invoke("connect_account", { agent: c.dataset.connect });
      toast("請在跳出的視窗中登入");
      return;
    }
    const d = e.target.closest("[data-disconnect]");
    if (d) invoke("disconnect_account", { agent: d.dataset.disconnect });
  });
  $("#clearAccounts").addEventListener("click", () => {
    invoke("clear_accounts");
    toast("已清除登入資料");
  });
  $("#reloadLog").addEventListener("click", loadLog);
  $("#copyDiag").addEventListener("click", copyDiag);
}

function syncControls() {
  $$(".seg").forEach((seg) => {
    $$("button", seg).forEach((b) => b.setAttribute("aria-pressed", String(config[seg.dataset.key] === b.value)));
  });
  $$('input[type="checkbox"][data-key]').forEach((cb) => (cb.checked = !!config[cb.dataset.key]));
  if (document.activeElement !== $("#offset")) $("#offset").value = config.offset_x;
  $("#refreshMin").value = String(config.refresh_minutes);
}

// ───────── 用量 ─────────
function bar(w, weekly) {
  const r = left(w);
  const reset = w.resets_at ? `${resetText(w.resets_at)} 重置 · 還有 ${untilText(w.resets_at)}` : "";
  return `<div class="bar">
    <div class="bar-top"><span>${w.label}</span><b>剩 ${Math.round(r)}%</b></div>
    <div class="bar-track"><div class="bar-fill ${weekly ? "wk" : "lv-" + level(r)}" style="width:${r}%"></div></div>
    <small>${reset}</small>
  </div>`;
}

function connectButton(agent, primary = true) {
  const a = AGENTS[agent];
  return `<button class="btn${primary ? " primary" : ""}" data-connect="${agent}">連結 ${a.account}</button>`;
}

function renderUsage() {
  const cards = $("#cards");
  const shown = state.quotas.filter((q) => q.installed);
  $("#refresh").disabled = state.refreshing;
  $("#refresh").textContent = state.refreshing ? "更新中…" : "立即更新";
  const newest = Math.max(0, ...state.quotas.filter((q) => q.ok).map((q) => q.updated));
  $("#usageHint").textContent = newest
    ? `上次更新 ${new Date(newest * 1000).toLocaleTimeString("zh-TW", { hour12: false })}，每 ${config.refresh_minutes} 分鐘自動更新。`
    : state.refreshing ? "正在讀取各家 AI 的額度…" : "";
  if (!shown.length) {
    cards.innerHTML = state.refreshing && !state.quotas.length
      ? `<div class="empty">正在偵測…</div>`
      : `<article class="card welcome">
          <b>連結你的 AI 帳號</b>
          <p>登入後，工作列就會顯示 5 小時與每週額度。登入是在官方網站上完成，DW Bar 不會看到你的密碼。</p>
          <div class="acts">${connectButton("claude")}${connectButton("codex", false)}</div>
        </article>`;
    return;
  }
  cards.innerHTML = shown
    .map((q) => {
      let body;
      if (q.ok) body = q.windows.map((w) => bar(w, w.period === "week")).join("");
      else {
        const cta = q.connected
          ? `<button class="btn" data-connect="${q.agent}">重新登入</button>`
          : connectButton(q.agent);
        body = `<div class="notice">${escapeHtml(q.notice || "沒有資料")}</div><div class="acts">${cta}</div>`;
      }
      const src = q.source === "account" ? "帳號" : q.source === "cli" ? "命令列" : "";
      const meta = [
        q.plan && `<span>方案 <b>${escapeHtml(q.plan)}</b></span>`,
        `<span>餘額 <b>${q.balance ? escapeHtml(q.balance) : "—"}</b></span>`,
        src && `<span class="src">來源 ${src}</span>`,
      ].filter(Boolean).join("");
      return `<article class="card">
        <div class="card-head"><span class="ring">${ringSVG(q.agent, q)}</span>
          <div><b>${escapeHtml(q.name)}</b><span>${escapeHtml(q.account || "")}</span></div></div>
        ${body}
        <div class="meta">${meta}</div>
      </article>`;
    })
    .join("");
}

// ───────── AI 帳號 ─────────
function renderAgents() {
  const box = $("#agentRows");
  if (box.dataset.built !== "1") {
    box.innerHTML = Object.entries(AGENTS)
      .filter(([id]) => id !== "dw")
      .map(([id, a]) => `<div class="row agent" data-agent="${id}">
        <span class="ring" style="width:40px;height:40px">${ringSVG(id, null)}</span>
        <div class="lbl">
          <b>${a.name}</b>
          <span class="acct"></span>
          <span class="where"></span>
          <details><summary>進階：指定命令列工具路徑</summary>
            <input class="path" placeholder="選填，例如 C:\\Users\\你\\.local\\bin\\${id === "claude" ? "claude" : "codex"}.exe" spellcheck="false" />
          </details>
        </div>
        <div class="acts col"><span class="link-btn"></span>
          <label class="switch" title="在工作列顯示"><input type="checkbox" /><i></i></label></div>
      </div>`)
      .join("");
    $$(".row", box).forEach((row) => {
      const id = row.dataset.agent;
      $('input[type="checkbox"]', row).addEventListener("change", (e) =>
        save({ [id]: { ...config[id], enabled: e.target.checked } }),
      );
      $(".path", row).addEventListener("change", (e) =>
        save({ [id]: { ...config[id], custom_path: e.target.value.trim() || null } }),
      );
    });
    box.dataset.built = "1";
  }
  $$(".row", box).forEach((row) => {
    const id = row.dataset.agent;
    const a = AGENTS[id];
    const q = state.quotas.find((x) => x.agent === id);
    const c = config[id];
    $('input[type="checkbox"]', row).checked = c.enabled;
    const path = $(".path", row);
    if (document.activeElement !== path) path.value = c.custom_path || "";
    const acct = $(".acct", row);
    if (c.connected) {
      acct.innerHTML = `<i class="dot ok"></i>已連結 ${escapeHtml(a.account)}${q?.account ? `（${escapeHtml(q.account)}）` : ""}`;
      $(".link-btn", row).innerHTML = `<button class="btn" data-disconnect="${id}">中斷連結</button>`;
    } else {
      acct.innerHTML = `<i class="dot"></i>尚未連結 ${escapeHtml(a.account)}（${escapeHtml(a.site)}）`;
      $(".link-btn", row).innerHTML = connectButton(id);
    }
    const found = [q?.app && "桌面 App", q?.cli && "命令列工具"].filter(Boolean);
    $(".where", row).textContent = found.length
      ? `這台電腦上有：${found.join("、")}`
      : "這台電腦上沒有偵測到它的 App，連結帳號就能顯示用量。";
  });
}

// ───────── 診斷 ─────────
function renderDiag() {
  const p = state.placement;
  const rect = (r) => (r ? `${r.left}, ${r.top} → ${r.right}, ${r.bottom}（${r.right - r.left}×${r.bottom - r.top}）` : "—");
  const rows = [
    ["版本", state.version],
    ["工作列", p.found_taskbar ? "已找到" : "找不到（Explorer 未就緒？）"],
    ["工作列對齊", p.centered ? "置中" : "靠左"],
    ["小工具按鈕", p.widgets ? "顯示" : "隱藏"],
    ["附著方式", p.embedded ? "嵌入工作列" : p.fallback ? "浮貼（嵌入被拒，已自動改用）" : "浮貼"],
    ["縮放", p.scale ? `${Math.round(p.scale * 100)}%` : "—"],
    ["工作列範圍", rect(p.taskbar)],
    ["膠囊範圍", rect(p.capsule)],
    ["全螢幕", state.fullscreen ? "是（膠囊已隱藏）" : "否"],
    ["設定檔", state.config_path],
    ["紀錄檔", state.log_path],
  ];
  $("#kv").innerHTML = rows.map(([k, v]) => `<dt>${k}</dt><dd>${escapeHtml(String(v))}</dd>`).join("");
  $("#placeHint").textContent = p.fallback
    ? "Windows 拒絕了嵌入，目前改用「浮貼在上方」。"
    : "";
}

async function loadLog() {
  const lines = await invoke("read_log", { lines: 200 });
  const log = $("#log");
  log.textContent = lines.join("\n") || "（還沒有紀錄）";
  log.scrollTop = log.scrollHeight;
}

async function copyDiag() {
  const lines = await invoke("read_log", { lines: 80 });
  const kv = $$("#kv dt").map((dt) => `${dt.textContent}: ${dt.nextElementSibling.textContent}`);
  const text = ["DW Bar 診斷", ...kv, "", ...lines].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    toast("已複製");
  } catch {
    toast("無法存取剪貼簿");
  }
}

// ───────── misc ─────────
function escapeHtml(s) {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

function renderAll() {
  config = structuredClone(state.config);
  applyTheme(config.theme);
  $("#ver").textContent = "v" + state.version;
  syncControls();
  renderUsage();
  renderAgents();
  renderDiag();
}

$("#brandRing").innerHTML = ringSVG("dw", { windows: [{ period: "five_hour", used_percent: 36 }, { period: "week", used_percent: 28 }] });
$("#legendRing").innerHTML = ringSVG("dw", { windows: [{ period: "five_hour", used_percent: 32 }, { period: "week", used_percent: 55 }] });
bindControls();
listen("state", (e) => {
  // Don't clobber an edit that is about to be saved with an older snapshot.
  if (saveTimer) return;
  state = e.payload;
  renderAll();
});
invoke("get_state").then((s) => {
  state = s;
  renderAll();
  goto(location.hash.slice(1) || "usage");
});
