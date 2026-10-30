// GET /download/store | /download/win-x64 | /download/win-arm64 | /download/win-msi | /download/mac
import { serve } from "../download.js";

export function onRequestGet({ params }) {
  return serve(params.os);
}
