// GET /api/latest → { version, date, sizes: { "win-x64", "win-arm64", "mac" } } of the newest
// release, for the download buttons. `size` stays for older pages (Windows x64).
//
// Reads the public release redirect and file headers instead of api.github.com, whose
// unauthenticated limit is shared by everyone on the same server address.
const REPO = "https://github.com/Axiang1211/DW_Bar/releases";
const NAMES = {
  "win-x64": "DW-Bar-Setup.exe",
  "win-arm64": "DW-Bar-Setup-arm64.exe",
  mac: "DW-Bar.dmg",
};
const CF = { cacheTtl: 600, cacheEverything: true };
const UA = { "User-Agent": "dwbar-site" };

async function version() {
  // /releases/latest → 302 /releases/tag/v0.8.0
  const r = await fetch(`${REPO}/latest`, { method: "HEAD", redirect: "manual", headers: UA, cf: CF });
  const m = (r.headers.get("Location") || "").match(/\/tag\/v?([^/?#]+)/);
  return m ? m[1] : null;
}

async function size(name) {
  try {
    const r = await fetch(`${REPO}/latest/download/${name}`, { method: "HEAD", redirect: "follow", headers: UA, cf: CF });
    const n = Number(r.headers.get("Content-Length"));
    return r.ok && n > 0 ? n : null;
  } catch {
    return null;
  }
}

export async function onRequestGet() {
  const keys = Object.keys(NAMES);
  const [v, ...list] = await Promise.all([version().catch(() => null), ...keys.map((k) => size(NAMES[k]))]);
  const sizes = Object.fromEntries(keys.map((k, i) => [k, list[i]]));
  return Response.json(
    { version: v, size: sizes["win-x64"], sizes },
    { headers: { "Cache-Control": `public, max-age=${v ? 600 : 60}` } },
  );
}
