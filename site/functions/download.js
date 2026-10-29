// GET /download/<os> (or /download?os=<os>), os = win-x64 | win-arm64 | win-msi | mac
// → the newest installer for that system, streamed
// from the latest GitHub release so the browser downloads it straight away on this site
// (no GitHub page in between). Without ?os it serves the Windows x64 installer.
const BASE = "https://github.com/Axiang1211/DW_Bar/releases/latest/download/";
const ASSETS = {
  "win-x64": "DW-Bar-Setup.exe",
  "win-arm64": "DW-Bar-Setup-arm64.exe",
  "win-msi": "DW-Bar.msi",
  mac: "DW-Bar.dmg",
};

export function onRequestGet({ request }) {
  return serve(new URL(request.url).searchParams.get("os"));
}

export async function serve(os) {
  const name = ASSETS[os] || ASSETS["win-x64"];
  const upstream = await fetch(BASE + name, {
    redirect: "follow",
    headers: { "User-Agent": "dwbar-site" },
    cf: { cacheTtl: 600, cacheEverything: true },
  });
  if (!upstream.ok || !upstream.body) {
    return new Response("安裝檔暫時無法下載，請稍後再試。", {
      status: 502,
      headers: { "Content-Type": "text/plain; charset=utf-8" },
    });
  }
  const headers = new Headers({
    "Content-Type": name.endsWith(".dmg") ? "application/x-apple-diskimage" : "application/octet-stream",
    "Content-Disposition": `attachment; filename="${name}"`,
    "Cache-Control": "public, max-age=600",
    "X-Content-Type-Options": "nosniff",
  });
  const len = upstream.headers.get("Content-Length");
  if (len) headers.set("Content-Length", len);
  return new Response(upstream.body, { status: 200, headers });
}
