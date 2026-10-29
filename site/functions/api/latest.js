// GET /api/latest → { version, size, date } of the newest release, for the download button.
export async function onRequestGet() {
  const r = await fetch("https://api.github.com/repos/Axiang1211/DW_Bar/releases/latest", {
    headers: { "User-Agent": "dwbar-site", Accept: "application/vnd.github+json" },
    cf: { cacheTtl: 600, cacheEverything: true },
  });
  if (!r.ok) return Response.json({ version: null }, { status: 200, headers: { "Cache-Control": "max-age=60" } });
  const rel = await r.json();
  const exe = (rel.assets || []).find((a) => a.name === "DW-Bar-Setup.exe");
  return Response.json(
    { version: (rel.tag_name || "").replace(/^v/, ""), size: exe ? exe.size : null, date: rel.published_at },
    { headers: { "Cache-Control": "public, max-age=600" } },
  );
}
