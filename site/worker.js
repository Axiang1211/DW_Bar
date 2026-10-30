// Cloudflare Worker entry (the dw-bar project is a Worker with static assets).
// /download[/<os>] and /api/latest run here; every other path is a static file.
import { serve } from "./functions/download.js";
import { onRequestGet as latest } from "./functions/api/latest.js";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname.replace(/\/+$/, "") || "/";
    if (path === "/download") return serve(url.searchParams.get("os"), request);
    const m = path.match(/^\/download\/([\w-]+)$/);
    if (m) return serve(m[1], request);
    if (path === "/api/latest") return latest();
    return env.ASSETS.fetch(request);
  },
};
