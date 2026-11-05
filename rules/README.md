# 線上規則（Online rules）

AI 網站改版、DW Bar 讀不到用量時，改這個檔案就能修好，**不用發新版、不用送商店審核**。

1. 編輯 `rules/rules.json`，把 `version` 加 1
2. 推送到 `main`（GitHub 網頁直接編輯也可以）
3. GitHub Actions「Sign online rules」會用簽章金鑰簽名，放到 `site/rules/v1/`，Cloudflare 自動上線
4. 用戶的 DW Bar 在啟動時與每 6 小時下載一次；簽章正確、版本較新才會使用，否則繼續用內建規則

## 格式

```json
{
  "schema": 1,
  "version": 2,
  "min_app": "1.1.0",
  "services": {
    "claude": {
      "probe_url": "https://claude.ai/api/organizations",
      "script": "window.__dwbar = null; (async () => { ... window.__dwbar = { done: true, ok: true, data: {...} }; })();"
    }
  }
}
```

每個服務（`claude`、`codex`、`gemini`、`cursor`、`windsurf`、`deepseek`）可以覆蓋這些欄位，沒寫的沿用程式內建值：

| 欄位 | 說明 |
| --- | --- |
| `login_url` | 連結帳號時開啟的登入頁 |
| `probe_url` | 讀取用量前，隱藏視窗先載入的同網域頁面 |
| `host` | 網站網域 |
| `flow` | 仍屬於登入流程的路徑（陣列） |
| `label` | 顯示名稱 |
| `script` | 在 `probe_url` 頁面執行的讀取腳本。成功：`window.__dwbar = { done: true, ok: true, data }`；沒登入：`{ done: true, ok: false, error: "not_signed_in" }`；其他錯誤：`{ done: true, ok: false, error: "原因" }` |

`data` 可以是該服務原本的回應（交給程式內建的解析器），也可以直接是**標準格式**，這樣網站 API 改了也不用改程式：

```json
{
  "dwbar": 1,
  "plan": "Pro",
  "account": "you@example.com",
  "balance": "$12.30",
  "windows": [
    { "period": "five_hour", "used_percent": 32, "resets_at": 1767225600 },
    { "period": "week", "remaining_percent": 59, "resets_at": "2026-11-12T03:00:00Z", "label": "每週（全部模型）" }
  ]
}
```

`period`：`five_hour`、`day`、`week`、`month`、`other`。`resets_at` 可以是 Unix 秒數或 ISO 8601。`label` 不寫時用「5 小時」「每週」等預設名稱（會自動翻譯）。

`min_app`：需要的最低 DW Bar 版本；較舊的版本會忽略這份規則。

商店政策：線上規則只用來修正既有服務的讀取方式，不拿來增加新功能。
