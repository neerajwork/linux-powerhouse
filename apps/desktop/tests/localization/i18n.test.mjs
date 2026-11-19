import test from "node:test";
import assert from "node:assert/strict";
import i18next from "i18next";

import en from "../../src/localization/resources/en.json" with { type: "json" };
import kn from "../../src/localization/resources/kn.json" with { type: "json" };
import enCommon from "../../src/localization/resources/en/common.json" with { type: "json" };
import knCommon from "../../src/localization/resources/kn/common.json" with { type: "json" };

const resources = {
  en: {
    translation: en,
    common: enCommon,
  },
  kn: {
    translation: kn,
    common: knCommon,
  },
};

test("i18next resolves English application and common translations", async () => {
  const instance = i18next.createInstance();

  await instance.init({
    resources,
    lng: "en",
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: ["translation", "common"],
    defaultNS: "translation",
  });

  assert.equal(instance.language, "en");
  assert.equal(instance.t("app.name"), "Linux Powerhouse");
  assert.equal(
    instance.t("app.tagline"),
    "Powerful Linux. Simplified.",
  );
  assert.equal(
    instance.t("actions.save", { ns: "common" }),
    "Save",
  );
});

test("i18next switches from English to Kannada translations", async () => {
  const instance = i18next.createInstance();

  await instance.init({
    resources,
    lng: "en",
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: ["translation", "common"],
    defaultNS: "translation",
  });

  await instance.changeLanguage("kn");

  assert.equal(instance.language, "kn");
  assert.equal(
    instance.t("app.tagline"),
    "ಶಕ್ತಿಯುತ ಲಿನಕ್ಸ್. ಸರಳೀಕೃತ.",
  );
  assert.equal(
    instance.t("actions.save", { ns: "common" }),
    "ಉಳಿಸಿ",
  );
  assert.equal(
    instance.t("status.loading", { ns: "common" }),
    "ಲೋಡ್ ಆಗುತ್ತಿದೆ",
  );
});

test("i18next falls back to English when a Kannada translation is missing", async () => {
  const instance = i18next.createInstance();

  const fallbackResources = {
    ...resources,
    kn: {
      ...resources.kn,
      translation: {
        app: {
          name: "Linux Powerhouse",
        },
      },
    },
  };

  await instance.init({
    resources: fallbackResources,
    lng: "kn",
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: ["translation", "common"],
    defaultNS: "translation",
  });

  assert.equal(instance.language, "kn");
  assert.equal(
    instance.t("app.tagline"),
    "Powerful Linux. Simplified.",
  );
});

test("i18next falls back to English for missing common namespace translations", async () => {
  const instance = i18next.createInstance();

  const fallbackResources = {
    ...resources,
    kn: {
      ...resources.kn,
      common: {
        actions: {
          save: "ಉಳಿಸಿ",
        },
      },
    },
  };

  await instance.init({
    resources: fallbackResources,
    lng: "kn",
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: ["translation", "common"],
    defaultNS: "translation",
  });

  assert.equal(
    instance.t("status.loading", { ns: "common" }),
    "Loading",
  );
});

test("i18next switches from Kannada back to English", async () => {
  const instance = i18next.createInstance();

  await instance.init({
    resources,
    lng: "kn",
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: ["translation", "common"],
    defaultNS: "translation",
  });

  assert.equal(instance.t("app.tagline"), "ಶಕ್ತಿಯುತ ಲಿನಕ್ಸ್. ಸರಳೀಕೃತ.");

  await instance.changeLanguage("en");

  assert.equal(instance.language, "en");
  assert.equal(instance.t("app.tagline"), "Powerful Linux. Simplified.");
  assert.equal(
    instance.t("actions.save", { ns: "common" }),
    "Save",
  );
});

test("i18next falls back to English for an unsupported language", async () => {
  const instance = i18next.createInstance();

  await instance.init({
    resources,
    lng: "fr",
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: ["translation", "common"],
    defaultNS: "translation",
  });

  assert.equal(instance.resolvedLanguage, "en");
  assert.equal(instance.t("app.tagline"), "Powerful Linux. Simplified.");
  assert.equal(
    instance.t("actions.save", { ns: "common" }),
    "Save",
  );
});
