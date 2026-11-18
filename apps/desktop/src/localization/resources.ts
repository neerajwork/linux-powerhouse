import en from "./resources/en.json";
import kn from "./resources/kn.json";
import enCommon from "./resources/en/common.json";
import knCommon from "./resources/kn/common.json";

export const localizationResources = {
  en: {
    translation: en,
    common: enCommon,
  },
  kn: {
    translation: kn,
    common: knCommon,
  },
};

export const localizationNamespaces = ["translation", "common"] as const;
export const defaultLocalizationNamespace = "translation";
