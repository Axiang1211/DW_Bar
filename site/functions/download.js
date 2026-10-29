// GET /download → the newest installer, streamed from the latest GitHub release so the
// browser downloads it straight away on this site (no GitHub page in between).
const ASSET = "https://github.com/Axiang1211/DW_Bar/releases/latest/download/DW-Bar-Setup.exe";

export async function onRequestGet() {
  const upstream = await fetch(ASSET, {
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
    "Content-Type": "application/octet-stream",
    "Content-Disposition": 'attachment; filename="DW-Bar-Setup.exe"',
    "Cache-Control": "public, max-age=600",
    "X-Content-Type-Options": "nosniff",
  });
  const len = upstream.headers.get("Content-Length");
  if (len) headers.set("Content-Length", len);
  return new Response(upstream.body, { status: 200, headers });
}
