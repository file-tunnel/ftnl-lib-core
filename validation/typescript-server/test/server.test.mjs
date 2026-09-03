import assert from "node:assert/strict";
import test from "node:test";
import { InternalCommandSchema, ServerRequestContextSchema, TrustedActorSchema } from "../dist/server.js";
const actor = { userId: "user-1", tenantId: "tenant-1", roles: ["tunnel-writer"] };
const context = { requestId: "req-1", traceId: "trace-1", actor, sourceIp: "127.0.0.1" };

test("accepts bounded server values", () => {
  assert.deepEqual(TrustedActorSchema.parse({ userId: " user-1 ", roles: [] }), { userId: " user-1 ", roles: [] });
  assert.deepEqual(ServerRequestContextSchema.parse(context), context);
  const command = { operationId: "tunnels.create", idempotencyKey: "idem-1", context, payload: {} };
  assert.deepEqual(InternalCommandSchema.parse(command), command);
});
for (const [schema, label, value] of [
  [TrustedActorSchema, "empty user", { userId: "", roles: [] }], [TrustedActorSchema, "long user", { userId: "u".repeat(129), roles: [] }],
  [TrustedActorSchema, "empty role", { userId: "user-1", roles: [""] }], [TrustedActorSchema, "too many roles", { userId: "user-1", roles: Array.from({ length: 65 }, () => "tunnel-writer") }],
  [TrustedActorSchema, "credential leak", { userId: "user-1", roles: [], credential: "secret" }],
  [ServerRequestContextSchema, "bad IP", { ...context, sourceIp: "not-an-ip" }], [ServerRequestContextSchema, "bad nested request", { ...context, requestId: "" }],
  [ServerRequestContextSchema, "client identity", { ...context, userId: "client-supplied" }],
  [InternalCommandSchema, "missing operation", { context, payload: {} }], [InternalCommandSchema, "empty operation", { operationId: "", context, payload: {} }],
  [InternalCommandSchema, "long operation", { operationId: "o".repeat(257), context, payload: {} }],
  [InternalCommandSchema, "long idempotency", { operationId: "tunnels.create", idempotencyKey: "i".repeat(129), context, payload: {} }],
  [InternalCommandSchema, "unknown field", { operationId: "tunnels.create", context, payload: {}, token: "secret" }],
]) test(`rejects server value: ${label}`, () => assert.equal(schema.safeParse(value).success, false));
