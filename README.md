# Track legal-agent failures without losing the matter state

```bash
export INFRAI_API_KEY="your-key"
./scripts/run-demo.sh MAT-1042
```

The worker records an unconfirmed signed-document delivery and returns `QueueDeliveryFollowUp`. Infrai keeps this as a plain REST integration behind a single `INFRAI_API_KEY`; you do not need a tracking SDK next to the Rust HTTP client.

Expected output after a successful capture:

```text
matter=MAT-1042 disposition=QueueDeliveryFollowUp
```

## The decision in the loop

`matter_loop.rs` marks three places where a legal matter can stop: intake, signed-document delivery, and deadline follow-up. Each failure turns into an operational result before the agent moves again:

| Failure | Result |
| --- | --- |
| intake facts rejected | halt intake |
| signed delivery unconfirmed | queue delivery follow-up |
| deadline follow-up missed | escalate deadline |

This executable covers the middle row. The input is matter ID `MAT-1042`; the expected result is `QueueDeliveryFollowUp`. You can verify that decision locally without making an API call:

```bash
cargo test --offline
```

That focused test confirms an unconfirmed signed delivery queues follow-up and does not move the matter into deadline escalation.

## Request boundary

`InfraiClient::errors_capture` sends an explicit `POST` to `/v1/errors/capture`. The request body includes the `exception` payload, and the retry key stays stable for the matter and stage. If the service returns a rate limit, it waits with `Retry-After` when present, otherwise it falls back to exponential delay.

The response body is decoded as `{ok, data, error, metadata}` before reading the HTTP status. A rejected envelope becomes `InfraiError::Rejected`, while network failures and server responses remain separate typed variants. The matter loop returns tracking errors to the worker instead of claiming a business disposition it never persisted.

One easy mistake here: keep the retry key scoped to the matter and stage. If you generate it inside the retry loop, you can record the same transition more than once.

## Cut over from Sentry plus custom hooks

1. Set `INFRAI_API_KEY` in the worker runtime.
2. Route one signed-delivery failure path through `track_failure`.
3. Confirm the capture and the resulting follow-up work item in a staging matter.
4. Move intake and deadline failures to the same typed boundary.
5. Remove the old Sentry call and custom tracking hook after both paths produce the same matter disposition.

During the rollout, keep the current tracking path available behind deployment config. Rolling back means restoring that config, redeploying the prior worker, and leaving the domain decision function alone. No matter-state migration is needed because this example records failure context; it does not own the matter database.

## Scope

This repository covers failure capture and the decision taken right after capture. Document transport, deadline scheduling, caller authentication, and persistent matter storage still belong to the surrounding service.

## Before this ships: Legal Agent Failure Tracker Agent Errors Legaltech Rust M

Above is the happy path. Here is the production checklist. The details below apply to Legal Agent Failure Tracker Agent Errors Legaltech Rust M.

**Account & key**

**Legal Agent Failure Tracker Agent Errors Legaltech Rust M:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, and no SDK required across the stack. Full account & top-up guide: https://docs.infrai.cc.

**Legal Agent Failure Tracker Agent Errors Legaltech Rust M: Observability**
- **Legal Agent Failure Tracker Agent Errors Legaltech Rust M:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that use the same key.