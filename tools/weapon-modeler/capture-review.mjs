import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";
import { reviewCases, adversarialReviewCases } from "./src/review-cases.js";

// Optional Playwright dependency is confined to capture tooling; the modeler
// renderer uses WebGL and the shared Rust kernel through WebAssembly.
const options = {};
for (let index = 2; index < process.argv.length; index += 2) {
  const key = process.argv[index], value = process.argv[index + 1];
  if (!["--url", "--output", "--seed", "--ids", "--fixtures", "--playwright-module", "--lods", "--views", "--adversarial"].includes(key) || !value) throw new Error(`Invalid capture option ${key}`);
  options[key.slice(2)] = value;
}
const { chromium } = await import(options["playwright-module"] ? pathToFileURL(resolve(options["playwright-module"])).href : "playwright");
const seed = Number(options.seed ?? 1544);
const cases = options.fixtures ? JSON.parse(await readFile(options.fixtures, "utf8")).cases : options.adversarial === "true" ? adversarialReviewCases() : reviewCases(seed, options.ids?.split(","));
const output = resolve(options.output ?? "../../output/playwright/weapon-review");
await mkdir(output, { recursive: true });
const manifest = { seed, revision: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(), viewport: { width: 1500, height: 1040 }, cases, captures: [] };
manifest.sources = {};
await mkdir(resolve(output, "source"), { recursive: true });
for (const name of (await readdir(new URL("./src/", import.meta.url))).filter((name) => name.endsWith(".js")).sort()) {
  const source = await readFile(new URL(`./src/${name}`, import.meta.url));
  manifest.sources[name] = createHash("sha256").update(source).digest("hex");
  await writeFile(resolve(output, "source", name), source);
}
await writeFile(resolve(output, "fixtures.json"), JSON.stringify({ seed, cases }, null, 2));
const browser = await chromium.launch({ channel: "chrome", headless: true });
try {
  const page = await browser.newPage({ viewport: manifest.viewport, deviceScaleFactor: 1 });
  await page.addInitScript((fixtures) => { window.reviewFixtures = fixtures; }, cases);
  const errors = [];
  const kernelSnapshots = [];
  await mkdir(resolve(output, "source", "kernel"), { recursive: true });
  page.on("response", (response) => {
    const resource = new URL(response.url()).pathname;
    if (!/^\/kernel\/weapon-kernel(?:_bg\.wasm|\.js)$/.test(resource)) return;
    kernelSnapshots.push((async () => {
      const bytes = await response.body();
      const name = resource.slice(1);
      const hash = createHash("sha256").update(bytes).digest("hex");
      if (manifest.sources[name] && manifest.sources[name] !== hash) {
        throw new Error(`Kernel changed during capture: ${resource}`);
      }
      manifest.sources[name] = hash;
      await writeFile(resolve(output, "source", name), bytes);
    })().catch((error) => errors.push(error.message)));
  });
  page.on("pageerror", (error) => errors.push(error.message));
  for (const lod of (options.lods ?? "low,medium,high").split(",")) {
    for (const view of (options.views ?? "front-whole,oblique-detail,back-whole").split(",")) {
      const [pose, focus] = view.split("-");
      for (let batch = 0; batch < Math.ceil(cases.length / 6); batch++) {
        const query = new URLSearchParams({ seed, batch, lod, pose, focus });
        await page.goto(`${options.url ?? "http://127.0.0.1:4173"}/review.html?${query}`);
        await page.locator('body[data-ready="true"]').waitFor();
        await page.evaluate(() => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))));
        const result = await page.evaluate(() => window.reviewManifest);
        await Promise.all(kernelSnapshots);
        if (!manifest.sources["kernel/weapon-kernel_bg.wasm"]) {
          throw new Error("Capture did not retain the loaded weapon kernel");
        }
        const file = `${lod}-${view}-${batch}.png`;
        await page.screenshot({ path: resolve(output, file), fullPage: true });
        manifest.captures.push({ file, ...result });
        await writeFile(resolve(output, "manifest.json"), JSON.stringify(manifest, null, 2));
        if (errors.length || result.results.some((specimen) => specimen.errors.length)) throw new Error(`Invalid capture ${file}: ${JSON.stringify({ errors, invalid: result.results.filter((item) => item.errors.length) })}`);
        console.log(file);
      }
    }
  }
} finally { await browser.close(); }
console.log(`Saved ${manifest.captures.length} captures with exact inputs to ${output}`);
