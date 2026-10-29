// GET /api/latest → { version, date, sizes: { "win-x64", "win-arm64", "mac" } } of the newest
// release, for the download buttons. `size` stays for older pages (Windows x64).
const NAMES = {
  "win-x64": "DW-Bar-Setup.exe",
  "win-arm64": "DW-Bar-Setup-arm64.exe",
  mac: "DW-Bar.dmg",
};

export async function onRequestGet() {
  const r = await fetch("https://api.github.com/repos/Axiang1211/DW_Bar/releases/latest", {
    headers: { "User-Agent": "dwbar-site", Accept: "application/vnd.github+json" },
    cf: { cacheTtl: 600, cacheEverything: true },
  });
  if (!r.ok) return Response.json({ version: null }, { status: 200, headers: { "Cache-Control": "max-age=60" } });
  const rel = await r.json();
  const sizes = {};
  for (const [key, name] of Object.entries(NAMES)) {
    const a = (rel.assets || []).find((x) => x.name === name);
    sizes[key] = a ? a.size : null;
  }
  return Response.json(
    { version: (rel.tag_name || "").replace(/^v/, ""), size: sizes["win-x64"], sizes, date: rel.published_at },
    { headers: { "Cache-Control": "public, max-age=600" } },
  );
}
