import { chromium } from "playwright";

const logs = [];
const browser = await chromium.launch({
  headless: true,
  args: [
    "--use-gl=angle",
    "--use-angle=swiftshader",
    "--enable-webgl",
    "--ignore-gpu-blocklist",
    "--enable-unsafe-swiftshader",
  ],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
page.on("console", (m) => logs.push(`${m.type()}: ${m.text()}`));
page.on("pageerror", (e) => logs.push(`pageerror: ${e}`));
await page.goto("http://127.0.0.1:8080/", { waitUntil: "domcontentloaded", timeout: 60000 });
await page.waitForTimeout(5000);
console.log("title", await page.title());
console.log("canvas", await page.locator("canvas").count());
console.log("boot", await page.locator("#boot").count());
await page.screenshot({ path: "/workspace/screenshots/title.png" });
await page.mouse.click(1280 * 0.2, 800 * 0.76);
await page.waitForTimeout(900);
await page.screenshot({ path: "/workspace/screenshots/intro.png" });
await page.mouse.click(640, 400);
await page.waitForTimeout(900);
await page.screenshot({ path: "/workspace/screenshots/world.png" });
await page.mouse.click(1280 * 0.28, 800 * 0.58);
await page.waitForTimeout(900);
await page.screenshot({ path: "/workspace/screenshots/town.png" });
await page.mouse.click(1280 * 0.16, 800 * 0.82);
await page.waitForTimeout(1600);
await page.screenshot({ path: "/workspace/screenshots/combat.png" });
console.log("--- logs ---");
for (const line of logs.slice(-50)) console.log(line);
await browser.close();
