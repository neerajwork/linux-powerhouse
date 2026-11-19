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

test("English application resources provide application name and tagline", async () => {
  const app = await loadResource("en/app.json");

  assert.equal(app.name, "Linux Powerhouse");
  assert.equal(app.tagline, "Powerful Linux. Simplified.");
});

test("Kannada application resources provide application name and tagline", async () => {
  const app = await loadResource("kn/app.json");

  assert.equal(app.name, "Linux Powerhouse");
  assert.equal(app.tagline, "ಶಕ್ತಿಯುತ ಲಿನಕ್ಸ್. ಸರಳೀಕೃತ.");
});

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

test("English application resources provide complete shell translations", async () => {
  const app = await loadResource("en/app.json");

  assert.equal(app.navigation.dashboard, "Dashboard");
  assert.equal(app.navigation.systemHealth, "System Health");
  assert.equal(app.navigation.healthWorkspace, "Health Workspace");
  assert.equal(app.navigation.actionAudit, "Action Audit");
  assert.equal(app.navigation.services, "Services");
  assert.equal(app.navigation.security, "Security");
  assert.equal(app.navigation.aiAssistant, "AI Assistant");

  assert.equal(app.sections.dashboard.title, "Your Linux system, at a glance.");
  assert.equal(app.sections.monitoring.title, "Monitoring");
  assert.equal(app.sections.alertHistory.title, "Alert History");

  assert.equal(app.actions.refresh, "Refresh");
  assert.equal(app.actions.refreshing, "Refreshing…");
  assert.equal(
    app.errors.readSystemData,
    "Unable to read system data: {{error}}",
  );
});

test("Kannada application resources provide complete shell translations", async () => {
  const app = await loadResource("kn/app.json");

  assert.equal(app.navigation.dashboard, "ಡ್ಯಾಶ್‌ಬೋರ್ಡ್");
  assert.equal(app.navigation.systemHealth, "ಸಿಸ್ಟಮ್ ಆರೋಗ್ಯ");
  assert.equal(app.navigation.healthWorkspace, "ಆರೋಗ್ಯ ಕಾರ್ಯಕ್ಷೇತ್ರ");
  assert.equal(app.navigation.actionAudit, "ಕ್ರಿಯಾ ಲೆಕ್ಕಪರಿಶೋಧನೆ");
  assert.equal(app.navigation.services, "ಸೇವೆಗಳು");
  assert.equal(app.navigation.security, "ಭದ್ರತೆ");
  assert.equal(app.navigation.aiAssistant, "AI ಸಹಾಯಕ");

  assert.equal(app.sections.dashboard.title, "ನಿಮ್ಮ Linux ವ್ಯವಸ್ಥೆ — ಒಂದು ನೋಟದಲ್ಲಿ.");
  assert.equal(app.sections.monitoring.title, "ಮೇಲ್ವಿಚಾರಣೆ");
  assert.equal(app.sections.alertHistory.title, "ಎಚ್ಚರಿಕೆ ಇತಿಹಾಸ");

  assert.equal(app.actions.refresh, "ನವೀಕರಿಸಿ");
  assert.equal(app.actions.refreshing, "ನವೀಕರಿಸಲಾಗುತ್ತಿದೆ…");
  assert.ok(app.errors.readSystemData.includes("{{error}}"));
});
