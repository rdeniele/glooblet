import { isErrorPayload } from "./types";

/**
 * Extracts a human-readable message from whatever a failed `invoke()` call
 * rejects with. Tauri commands that return `Result<T, AppError>` reject the
 * JS promise with the serialized `{ code, message }` object (see
 * src-tauri/src/error.rs), not a string — so `String(e)` on it produces the
 * useless "[object Object]" instead of the real reason. This is the one
 * place that unwraps it correctly; every toast.error(...) call should route
 * its caught value through this instead of String(e).
 */
export function getErrorMessage(e: unknown): string {
  if (isErrorPayload(e)) return e.message;
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}
