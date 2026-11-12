import test from "node:test";
import assert from "node:assert/strict";

import {
  normalizePosixLocale,
  resolveSupportedLocale,
} from "../../src/localization/locale.ts";

test("normalizes language, territory, and encoding", () => {
  assert.equal(normalizePosixLocale("en_US.UTF-8"), "en-US");
});

test("normalizes a language-only locale", () => {
  assert.equal(normalizePosixLocale("en"), "en");
});

test("normalizes a language and territory without encoding", () => {
  assert.equal(normalizePosixLocale("kn_IN"), "kn-IN");
});

test("normalizes Kannada locale with encoding", () => {
  assert.equal(normalizePosixLocale("kn_IN.UTF-8"), "kn-IN");
});

test("normalizes an explicit script and territory", () => {
  assert.equal(normalizePosixLocale("sr_Latn_RS"), "sr-Latn-RS");
});

test("rejects an empty locale identifier", () => {
  assert.equal(normalizePosixLocale(""), null);
});

test("rejects a whitespace-only locale identifier", () => {
  assert.equal(normalizePosixLocale("   "), null);
});

test("rejects a malformed locale identifier", () => {
  assert.equal(normalizePosixLocale("en_US_TOOLONGVARIANT"), null);
});

test("rejects an empty encoding suffix", () => {
  assert.equal(normalizePosixLocale("en_US."), null);
});

test("rejects multiple encoding separators", () => {
  assert.equal(normalizePosixLocale("en_US.UTF-8.ISO"), null);
});

test("rejects invalid characters in encoding suffix", () => {
  assert.equal(normalizePosixLocale("en_US.UTF-8!"), null);
});

test("normalizes a locale with a Latin script modifier", () => {
  assert.equal(normalizePosixLocale("sr_RS@latin"), "sr-Latn-RS");
});

test("normalizes a locale with encoding and Latin modifier", () => {
  assert.equal(
    normalizePosixLocale("sr_RS.UTF-8@latin"),
    "sr-Latn-RS",
  );
});

test("rejects a conflicting explicit script and Latin modifier", () => {
  assert.equal(
    normalizePosixLocale("sr_Cyrl_RS@latin"),
    null,
  );
});

test("rejects an unsupported locale modifier", () => {
  assert.equal(normalizePosixLocale("sr_RS@cyrillic"), null);
});

test("rejects an empty locale modifier", () => {
  assert.equal(normalizePosixLocale("sr_RS@"), null);
});

test("rejects multiple locale modifier separators", () => {
  assert.equal(normalizePosixLocale("sr_RS@latin@extra"), null);
});

test("rejects the C locale identifier during normalization", () => {
  assert.equal(normalizePosixLocale("C"), null);
});

test("rejects the POSIX locale identifier during normalization", () => {
  assert.equal(normalizePosixLocale("POSIX"), null);
});

test("rejects the C locale with an encoding suffix", () => {
  assert.equal(normalizePosixLocale("C.UTF-8"), null);
});

test("rejects the POSIX locale with an encoding suffix", () => {
  assert.equal(normalizePosixLocale("POSIX.UTF-8"), null);
});

test("rejects the C locale with a modifier", () => {
  assert.equal(normalizePosixLocale("C@latin"), null);
});

test("rejects the POSIX locale with a modifier", () => {
  assert.equal(normalizePosixLocale("POSIX@latin"), null);
});

test("normalizes a locale with an uppercase Latin modifier", () => {
  assert.equal(normalizePosixLocale("sr_RS@LATIN"), "sr-Latn-RS");
});

test("resolves regional English locale to en", () => {
  assert.equal(resolveSupportedLocale("en-IN"), "en");
});

test("resolves regional Kannada locale to kn", () => {
  assert.equal(resolveSupportedLocale("kn-IN"), "kn");
});

test("resolves unsupported language to English", () => {
  assert.equal(resolveSupportedLocale("fr-FR"), "en");
});

test("resolves a missing locale to English", () => {
  assert.equal(resolveSupportedLocale(null), "en");
});

test("resolves a malformed locale to English", () => {
  assert.equal(resolveSupportedLocale("kn--IN"), "en");
});

test("resolves an empty locale to English", () => {
  assert.equal(resolveSupportedLocale(""), "en");
});

test("resolves a whitespace-only locale to English", () => {
  assert.equal(resolveSupportedLocale("   "), "en");
});
