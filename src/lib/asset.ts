/** Public-file URL that respects Vite `base` (GitHub Pages lives at `/claymore-blade/`). */
export function asset(path: string): string {
  const p = path.replace(/^\//, "");
  return `${import.meta.env.BASE_URL}${p}`;
}
