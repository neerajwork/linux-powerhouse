import enApp from "./resources/en/app.json";
import knApp from "./resources/kn/app.json";
import enCommon from "./resources/en/common.json";
import knCommon from "./resources/kn/common.json";

export const localizationResources = {
  en: {
    app: enApp,
    common: enCommon,
  },
  kn: {
    app: knApp,
    common: knCommon,
  },
};

export const localizationNamespaces = ["app", "common"] as const;
export const defaultLocalizationNamespace = "app";
