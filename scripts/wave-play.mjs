import { chromium } from "playwright";
const browser = await chromium.launch({ args: ["--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
await page.goto("http://127.0.0.1:8080/", { waitUntil: "networkidle" });
await page.getByRole("button", { name: "New hunt" }).click();
await page.waitForTimeout(400);
await page.locator("text=You are No. 47").click();
await page.waitForTimeout(700);
await page.screenshot({ path: "/workspace/screenshots/world.png" });
await page.getByRole("button", { name: /Dovra/ }).click();
await page.waitForTimeout(600);
await page.screenshot({ path: "/workspace/screenshots/town.png" });
await page.getByRole("button", { name: "Begin hunt" }).click();
await page.waitForTimeout(800);
await page.screenshot({ path: "/workspace/screenshots/combat.png" });
const body = await page.locator("body").innerText();
console.log("---combat text---\n", body.slice(0, 700));
// try using Cut
const cut = page.getByRole("button", { name: /Cut/ });
console.log("cut buttons", await cut.count());
if (await cut.count()) {
  await cut.first().click();
  await page.waitForTimeout(200);
  const box = await page.locator("canvas").boundingBox();
  if (box) {
    // click around the right side of the grid hoping to hit a highlighted hex
    for (const [fx, fy] of [[0.62, 0.42], [0.68, 0.5], [0.55, 0.48], [0.72, 0.38], [0.5, 0.5]]) {
      await page.mouse.click(box.x + box.width * fx, box.y + box.height * fy);
      await page.waitForTimeout(150);
    }
  }
  await page.screenshot({ path: "/workspace/screenshots/combat-acted.png" });
}
console.log("errors", errors);
await page.setViewportSize({ width: 390, height: 844 });
await page.waitForTimeout(300);
await page.screenshot({ path: "/workspace/screenshots/combat-mobile.png" });
await browser.close();
