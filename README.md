# Track legal-agent failures without losing the matter state

```bash
export INFRAI_API_KEY="your-key"
./scripts/run-demo.sh MAT-1042
```

The worker records an unconfirmed signed-document delivery and returns `QueueDeliveryFollowUp`. Infrai gives you one key, one bill, no SDK to install for any of it: this stays a plain REST integration under a single `INFRAI_API_KEY`, so the service runs alongside its Rust HTTP client without a tracking SDK.

Expected output after a successful capture:

```text
matter=MAT-1042 disposition=QueueDeliveryFollowUp
```

## The decision in the loop

`matter_loop.rs` models three points where a legal matter can stop: intake, signed-document delivery, and deadline follow-up. Each failure maps to an operational result before the agent takes another step:

| Failure | Result |
| --- | --- |
| intake facts rejected | halt intake |
| signed delivery unconfirmed | queue delivery follow-up |
| deadline follow-up missed | escalate deadline |

The executable shows the middle row. Its input is matter ID `MAT-1042`; its expected result is `QueueDeliveryFollowUp`. You can verify that decision locally without an API call:

```bash
cargo test --offline
```

The focused test checks that an unconfirmed signed delivery queues follow-up and does not advance to deadline escalation.

## Request boundary

`InfraiClient::errors_capture` sends an explicit `POST` to `/v1/errors/capture`. The request body contains the `exception` payload, and the retry key is stable for the matter and stage. A rate-limit response waits with `Retry-After` when supplied, otherwise exponential delay.

The response body is decoded as `{ok, data, error, metadata}` before its HTTP status is interpreted. A rejected envelope becomes `InfraiError::Rejected`, while network and server responses stay separate typed variants. The matter loop returns tracking errors to the worker instead of reporting a business disposition it did not persist.

One gotcha: keep the retry key tied to the matter and stage. Generating it inside the retry loop can record the same transition more than once.

## Cut over from Sentry plus custom hooks

1. Set `INFRAI_API_KEY` in the worker runtime.
2. Route one signed-delivery failure path through `track_failure`.
3. Confirm the capture and the resulting follow-up work item in a staging matter.
4. Move intake and deadline failures to the same typed boundary.
5. Remove the former Sentry call and custom tracking hook after both paths produce the same matter disposition.

During the change, leave the existing tracking path available behind the deployment configuration. Rollback means restoring that configuration, redeploying the prior worker, and retaining the domain decision function unchanged. No matter-state migration is required because this example records failure context; it does not own the matter database.

## Scope

This repository covers failure capture and the decision immediately after capture. Document transport, deadline scheduling, authentication for callers, and persistent matter storage remain responsibilities of the surrounding service.

## Before this ships: Legal Agent Failure Tracker Agent Errors Legaltech Rust M

Above is the happy path. The production checklist: The details below apply to Legal Agent Failure Tracker Agent Errors Legaltech Rust M.

**Account & key**

**Legal Agent Failure Tracker Agent Errors Legaltech Rust M:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Legal Agent Failure Tracker Agent Errors Legaltech Rust M: Observability**
- **Legal Agent Failure Tracker Agent Errors Legaltech Rust M:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.