import assert from "node:assert/strict";
import test from "node:test";
import { parsePublic, safeParsePublic } from "../dist/public.js";
const problem = { type: "urn:test", title: "Invalid request", status: 400, requestId: "req-1" };

test("accepts contract boundaries and preserves exact identifiers", () => {
  const exact = { requestId: " req-1 ", traceId: " trace-1 ", locale: "en" };
  assert.deepEqual(parsePublic("request-meta", exact), exact);
  assert.equal(safeParsePublic("request-meta", { requestId: "r".repeat(128), traceId: "t".repeat(128), locale: "l".repeat(64) }).success, true);
  assert.deepEqual(parsePublic("page-query", { limit: 1 }), { limit: 1 });
  assert.deepEqual(parsePublic("page-query", { limit: 100, cursor: "c".repeat(512) }), { limit: 100, cursor: "c".repeat(512) });
  assert.equal(safeParsePublic("problem-details", { ...problem, status: 599, detail: "d".repeat(4096) }).success, true);
});
for (const [schema, label, value] of [
  ["request-meta", "missing trace", { requestId: "req-1" }], ["request-meta", "empty id", { requestId: "", traceId: "trace-1" }],
  ["request-meta", "long id", { requestId: "r".repeat(129), traceId: "trace-1" }], ["request-meta", "short locale", { requestId: "req-1", traceId: "trace-1", locale: "e" }],
  ["request-meta", "client identity", { requestId: "req-1", traceId: "trace-1", userId: "client-supplied" }],
  ["page-query", "missing limit", {}], ["page-query", "zero", { limit: 0 }], ["page-query", "high", { limit: 101 }], ["page-query", "fraction", { limit: 1.5 }],
  ["page-query", "empty cursor", { limit: 50, cursor: "" }], ["page-query", "unknown field", { limit: 50, offset: 1 }],
  ["problem-details", "low status", { ...problem, status: 399 }], ["problem-details", "high status", { ...problem, status: 600 }],
  ["problem-details", "fractional status", { ...problem, status: 400.5 }], ["problem-details", "empty title", { ...problem, title: "" }],
  ["problem-details", "long detail", { ...problem, detail: "d".repeat(4097) }], ["problem-details", "internal field", { ...problem, internalCode: "secret" }],
]) test(`rejects ${schema}: ${label}`, () => assert.equal(safeParsePublic(schema, value).success, false));
