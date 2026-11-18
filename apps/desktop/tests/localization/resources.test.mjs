import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const resourcePath = (path) =>
  fileURLToPath(
    new URL(`../../src/localization/resources/${path}`, import.meta.url),
  );

async function loadResource(path) {
  return JSON.parse(await readFile(resourcePath(path), "utf8"));
}

test("English common resources contain shared actions and statuses", async () => {
  const common = await loadResource("en/common.json");

  assert.equal(common.actions.cancel, "Cancel");
  assert.equal(common.actions.close, "Close");
  assert.equal(common.actions.save, "Save");
  assert.equal(common.status.loading, "Loading");
  assert.equal(common.status.unknown, "Unknown");
});

test("Kannada common resources contain shared actions and statuses", async () => {
  const common = await loadResource("kn/common.json");

  assert.equal(common.actions.cancel, "ರದ್ದುಮಾಡಿ");
  assert.equal(common.actions.close, "ಮುಚ್ಚಿ");
  assert.equal(common.actions.save, "ಉಳಿಸಿ");
  assert.equal(common.status.loading, "ಲೋಡ್ ಆಗುತ್ತಿದೆ");
  assert.equal(common.status.unknown, "ತಿಳಿದಿಲ್ಲ");
});

test("English and Kannada resources provide the application translation files", async () => {
  const [en, kn] = await Promise.all([
    loadResource("en.json"),
    loadResource("kn.json"),
  ]);

  assert.ok(Object.keys(en).length > 0);
  assert.ok(Object.keys(kn).length > 0);
});
