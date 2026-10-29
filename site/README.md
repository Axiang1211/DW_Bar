# dwbar.app.dreamsworld.top

DW Bar 的官方網站（靜態頁面 + Cloudflare Pages Functions）。

- `index.html` 介紹、使用方式、安裝須知、常見問題、下載
- `privacy.html`、`terms.html` 隱私權政策、服務條款
- `worker.js` + `wrangler.jsonc`：Cloudflare Worker。`/download/<win-x64|win-arm64|win-msi|mac>` 直接串流最新版安裝檔（網址不離開本站），`/api/latest` 提供版本與檔案大小；其他路徑是靜態檔
- `functions/`：實際的下載與版本程式（Worker 引用；若改用 Pages 專案也能直接當 Pages Functions）
- `_redirects`：備援，萬一程式沒在跑，直接導向安裝檔本身
- `.assetsignore`：不對外公開的檔案（程式碼、設定）

## 部署（Cloudflare Workers，連接 Git）

Cloudflare 後台 → Workers 和 Pages → `dw-bar` → 設定 → 建置：

- Git 存放庫 `Axiang1211/DW_Bar`，生產分支 `main`
- 組建命令 `npm run build`、部署命令 `npx wrangler deploy`、**根目錄 `site`**
- 自訂網域 `dwbar.app.dreamsworld.top`（安裝步驟用 `site/package-lock.json`，請一起提交）

之後每次推送到 `main`：GitHub Actions 會打包並發布新版本，Cloudflare 自動重新部署網站，下載按鈕自動提供最新版。
