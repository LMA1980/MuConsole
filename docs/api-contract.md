# MuConsole Read-Only API Contract

This document defines the read-only HTTP and Server-Sent Events (SSE) contract consumed by the `muconsole-data` Web Worker (`crates/muconsole-protocol/src/dto.rs` and `crates/muconsole-data/src/api.rs`).

All JSON field names and enum variants use `snake_case`, timestamps are Unix epoch milliseconds unless otherwise noted, and unknown fields are ignored by the client so upstream controllers can add fields without breaking compatibility. Optional fields may be either `null` or omitted.

## 1. Upstream Routing & Authentication

- **CI Endpoints (`ActionRunner-controller`)**:
  - `GET /api/repos`
  - `GET /api/repos/{owner}/{repo}/summary`
  - `GET /api/repos/{owner}/{repo}/topology`
  - `GET /api/events` (SSE stream)
  - `GET /api/auth/session` (operator session status)
  - `POST /api/auth/redeem` (one-time invite redemption)
  - `POST /api/auth/invites`, `GET /api/auth/invites`, `DELETE /api/auth/invites/{id}` (admin invite lifecycle)
  - Public repositories are readable anonymously. Private, internal, and unclassified repositories, raw job logs, and job restart mutations are only accessible when authenticated via either:
    1. **Invite-bound GitHub OAuth2 Session (`oauth2-proxy` + Nginx `auth_request`)**: Nginx validates the `_muconsole_oauth2` cookie against `http://127.0.0.1:4180/oauth2/auth`, extracts `X-Auth-Request-User` into `X-Forwarded-User: <github_login>`, and attaches the shared `X-Proxy-Secret` header. `ActionRunner-controller` verifies `X-Proxy-Secret` in constant time and grants operator access only if `<github_login>` has redeemed a one-time invite token (`invite_bound: true`).
    1. **API Key (`ACTION_RUNNER_LOG_API_KEY`)**: `Authorization: Bearer <key>` or `X-Api-Key: <key>` (fails closed when unset).
  - Anonymous requests or uninvited GitHub accounts receive `404` on private repositories and `401` / `403` on raw job logs and job restart mutations.
- **CD Endpoints (`MuConDe-controller`)**:
  - `GET /api/cd/deployments?repo={owner}/{repo}&limit={N}`
  - `GET /api/cd/deployments/{id}/gate`
  - `GET /api/cd/leases`
  - `GET /api/cd/events` (SSE stream)
  - Served on a dedicated read-only loopback listener (`MUCONDE_CD_API_ADDR`) behind the reverse proxy. Every `/api/cd/*` route requires `Authorization: Bearer <token>` loaded from `MUCONDE_CD_API_TOKEN_FILE` (trimmed, constant-time verified, fails closed with `401` if unset or empty).

______________________________________________________________________

## 1.5 Invite-Only GitHub Authentication Endpoints (`ActionRunner-controller`)

### `GET /api/auth/session`

Returns the caller's authentication and invite-binding state (`AuthSessionDto`). Safe to call anonymously on startup.

```json
{
  "authenticated": true,
  "github_login": "operator-one",
  "invite_bound": true,
  "role": "operator"
}
```

| State                       | `authenticated` | `github_login`   | `invite_bound` | `role`                    | UI `AuthState`              |
| --------------------------- | --------------- | ---------------- | -------------- | ------------------------- | --------------------------- |
| Anonymous                   | `false`         | `null`           | `false`        | `null`                    | `AuthState::Anonymous`      |
| GitHub signed in, uninvited | `false`         | `"guest-user"`   | `false`        | `null`                    | `AuthState::InviteRequired` |
| Invited operator / admin    | `true`          | `"operator-one"` | `true`         | `"operator"` \| `"admin"` | `AuthState::Operator`       |

### `POST /api/auth/redeem`

Redeems a one-time invite token (`?invite=<token>`) and permanently binds the caller's `X-Forwarded-User` GitHub login as an operator. Requires an active `oauth2-proxy` GitHub session (`auth_request /oauth2/auth`).

Request:

```json
{
  "token": "inv_019a4f8c_secret"
}
```

Response (`200 OK`):

```json
{
  "authenticated": true,
  "github_login": "operator-one",
  "invite_bound": true,
  "role": "operator"
}
```

### `POST /api/auth/invites` & `GET /api/auth/invites`

Admin-only endpoints for minting, listing, and revoking one-time operator invite tokens.

```json
{
  "role": "operator",
  "ttl_seconds": 86400,
  "note": "On-call operator invite"
}
```

______________________________________________________________________

## 2. CI REST Endpoints (`ActionRunner-controller`)

### `GET /api/repos`

Returns the repository registry, submodule graph, and latest default-branch CI metrics. `muconsole-data` accepts both the controller envelope (`{"fabric_rev", "authenticated", "repos": [...]}`) and a bare `[RepoDto]` array.

```json
{
  "fabric_rev": 482,
  "authenticated": false,
  "repos": [
    {
      "owner": "LMA1980",
      "repo": "superproject",
      "visibility": "public",
      "default_branch": "main",
      "latest_job_id": "job-1793563200-44444444",
      "score": 92,
      "coverage": 83.4,
      "bench_delta": -1.5,
      "status": "passing",
      "submodules": [
        {
          "path": "code/engine",
          "owner": "LMA1980",
          "repo": "engine",
          "pinned_sha": "4f2a9c1"
        }
      ],
      "parents": []
    },
    {
      "owner": "LMA1980",
      "repo": "engine",
      "visibility": "public",
      "default_branch": "main",
      "latest_job_id": null,
      "score": null,
      "coverage": null,
      "bench_delta": null,
      "status": "unknown",
      "submodules": [],
      "parents": [
        {
          "owner": "LMA1980",
          "repo": "superproject",
          "path": "code/engine"
        }
      ]
    }
  ]
}
```

| Field           | Type                                                     | Notes                                                                                    |
| --------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `visibility`    | `"public"` \| `"private"` \| `"internal"` \| `null`      | `"internal"` maps to `Visibility::Private`; `null` defaults when unclassified            |
| `latest_job_id` | `string` \| `u64` \| `null`                              | Controller string IDs (`"job-{ts}-{sha8}"`) map deterministically to monotonic `u64` IDs |
| `score`         | `u32` (`0..=100`) \| `null`                              | Quality score of the latest completed default-branch job                                 |
| `coverage`      | `f64` (`0.0..=100.0`) \| `null`                          | Line coverage in percent (converted to basis points `0..=10_000` in the store)           |
| `bench_delta`   | `f64` \| `null`                                          | Signed benchmark delta in percent vs. previous default-branch run                        |
| `status`        | `"unknown"` \| `"passing"` \| `"running"` \| `"failing"` | Default-branch CI state                                                                  |
| `submodules[]`  | `[{path, owner, repo, pinned_sha}]`                      | Parsed from `.gitmodules` and gitlink tree entries on the default branch                 |
| `parents[]`     | `[{owner, repo, path}]`                                  | Superprojects referencing this repository in `.gitmodules`                               |

### `GET /api/repos/{owner}/{repo}/summary[?branch=<ref>]`

Returns the CI scorecard and sparkline history for a repository.

```json
{
  "owner": "LMA1980",
  "repo": "superproject",
  "visibility": "public",
  "default_branch": "main",
  "branch": "main",
  "status": "passing",
  "latest_job_id": "job-1793563200-44444444",
  "score": 92,
  "letter_grade": "A",
  "coverage": 87.5,
  "coverage_bp": 8750,
  "bench_delta": 10.0,
  "history": [
    {
      "job_id": "job-1793563100-33333333",
      "commit_sha": "33333333",
      "status": "passed",
      "score": 88,
      "coverage": 80.0,
      "updated_at": "2026-11-01T19:50:00Z"
    }
  ]
}
```

### `GET /api/repos/{owner}/{repo}/topology`

Returns the railway station graph, active sidings (`branches[]`), and jobs (`trains[]`) for the Trainway panel.

```json
{
  "owner": "LMA1980",
  "repo": "superproject",
  "default_branch": "main",
  "fabric_rev": 482,
  "stations_source": "workflow",
  "stations": [
    { "id": "fmt-check", "name": "FMT", "depends_on": [] },
    { "id": "clippy-check", "name": "CLIPPY", "depends_on": ["fmt-check"] }
  ],
  "branches": [
    {
      "ref": "feat/rvv-gemm",
      "base_ref": "main",
      "fork_stage": "fmt-check",
      "pr_state": "open",
      "pr_number": 7,
      "head_sha": "77777777",
      "last_job": {
        "job_id": "job-1793563200-77777777",
        "status": "running",
        "commit_sha": "77777777",
        "stage": "clippy-check"
      }
    }
  ],
  "trains": [
    {
      "job_id": "job-1793563200-77777777",
      "ref": "feat/rvv-gemm",
      "status": "running",
      "commit_sha": "77777777",
      "stage": "clippy-check",
      "node_id": "node-a",
      "stage_states": [
        { "station": "fmt-check", "state": "passed", "node_id": null },
        { "station": "clippy-check", "state": "running", "node_id": "node-a" }
      ]
    }
  ]
}
```

- `stations[]`: accepts `"name"` (from `ActionRunner-controller`) or `"label"` (from fixtures).
- `branches[].pr_state`: `"none"` | `"open"` | `"draft"` (aliased to `PrState::Open`) | `"merged"` | `"closed"`.
- `branches[].last_job`: accepts an object `{"job_id": ...}`, a string `"job-..."`, a `u64`, or `null`.
- `trains[].stage_states`: accepts `[{"station", "state", "node_id"}]` objects or bare `"passed"` / `"running"` / `"pending"` / `"failed"` / `"skipped"` strings.

______________________________________________________________________

## 3. CI SSE Events (`GET /api/events`)

Additive event types emitted alongside existing `queue`, `jobs`, and `cluster` events:

| `event:`                  | `data:` JSON Payload                                                                                                                         |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `branch_created`          | `{"owner","repo","ref","base_ref","fork_stage","head_sha","fabric_rev"}`                                                                     |
| `branch_deleted`          | `{"owner","repo","ref","merged","fabric_rev"}`                                                                                               |
| `pr_state_changed`        | `{"owner","repo","ref","pr_number","pr_state","previous_state","base_ref","fabric_rev"}`                                                     |
| `job_stage_changed`       | `{"owner","repo","job_id","ref","stage","state","previous_state","node_id","fabric_rev"}`                                                    |
| `node_membership_changed` | `{"node_id","change":"joined"\|"left"\|"online"\|"offline","online_nodes":[...],"fabric_rev"}` (or `{"node_id","online":bool,"fabric_rev"}`) |

______________________________________________________________________

## 4. CD REST Endpoints (`MuConDe-controller`)

### `GET /api/cd/deployments?repo={owner}/{repo}&limit={N}`

Returns newest deployments first (`DeploymentDto`).

```json
[
  {
    "deployment_id": "dep-7f3c",
    "owner": "LMA1980",
    "repo": "superproject",
    "service": "model-service",
    "artifact_digest": "sha256:4be1",
    "target_node": "worker-riscv64",
    "active_slot": "b",
    "staged_slot": "b",
    "outcome": "promoted",
    "verdict": "promote",
    "it_passed": 2,
    "it_failed": 0,
    "it_total": 2,
    "critical_failed": 0,
    "rollback_reason": null,
    "tainted": false,
    "started_epoch_ms": 1793563000000,
    "finished_epoch_ms": 1793563012345
  }
]
```

### `GET /api/cd/deployments/{id}/gate`

Returns the live integration-test gate (`GateDto`), or `404` when unknown.

```json
{
  "deployment_id": "dep-7f3c",
  "owner": "LMA1980",
  "repo": "superproject",
  "service": "model-service",
  "artifact_digest": "sha256:4be1",
  "target_node": "worker-riscv64",
  "active_slot": "a",
  "staged_slot": "b",
  "restore_slot": "a",
  "lease_id": "lease-91d2",
  "lease_expires_epoch_ms": 1793563300000,
  "lease_ttl_remaining_ms": 287655,
  "outcome": "pending",
  "verdict": "pending",
  "rollback_reason": null,
  "rollback_detail": null,
  "tainted": false,
  "started_epoch_ms": 1793563000000,
  "finished_epoch_ms": null,
  "tests": [
    {
      "name": "systemd_unit_active",
      "critical": true,
      "state": "passed",
      "duration_ms": 512,
      "crate_name": null,
      "log_excerpt": null
    },
    {
      "name": "health_endpoint_ready",
      "critical": true,
      "state": "running",
      "duration_ms": 0,
      "crate_name": null,
      "log_excerpt": null
    }
  ]
}
```

- `outcome`: `"pending"` | `"promoted"` | `"rolled_back"` | `"tainted"`.
- `rollback_reason`: `"critical_test_failed"` | `"lease_expired"` | `"manual"` | `null`.
- `tests[].state`: `"pending"` | `"running"` | `"passed"` | `"failed"`.

### `GET /api/cd/leases`

Returns active TTL-bounded node deployment leases (`LeaseDto`).

```json
[
  {
    "lease_id": "lease-91d2",
    "deployment_id": "dep-7f3c",
    "target_node": "worker-riscv64",
    "service": "model-service",
    "artifact_digest": "sha256:4be1",
    "active_slot": "a",
    "candidate_slot": "b",
    "expires_epoch_ms": 1793563300000,
    "ttl_remaining_ms": 287655,
    "promoted": false
  }
]
```

______________________________________________________________________

## 5. CD SSE Events (`GET /api/cd/events`)

Emits `: heartbeat` comments every 15 s while idle and drops lagging consumers (> 1024 buffered events) so the worker reconnects and re-fetches the snapshot:

| `event:`        | `data:` JSON Payload                                                                  |
| --------------- | ------------------------------------------------------------------------------------- |
| `it_progress`   | `{"deployment_id","test_name","critical","state","duration_ms","timestamp_epoch_ms"}` |
| `promoted`      | `{"deployment_id","slot"}`                                                            |
| `rolled_back`   | `{"deployment_id","restore_slot","reason","detail"}`                                  |
| `tainted`       | `{"deployment_id","artifact_digest","reason"}`                                        |
| `lease_expired` | `{"deployment_id","lease_id"}`                                                        |
