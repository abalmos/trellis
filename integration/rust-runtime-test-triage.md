# Rust Runtime Matrix 0.12 Triage

This records the clean-break triage of every `rust: pending` row inherited by
the 0.12 development line. `KEEP` preserves the row as written, `REWRITE`
preserves the invariant but updates the row and test to native Rust-era
semantics, `MERGE` moves the assertion into the named retained row, and `RETIRE`
removes behavior that is not a current Rust runtime acceptance requirement.

## Jobs

| Action  | Case                                             | Disposition                                                                                                                |
| ------- | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| REWRITE | `jobs.keyed-jobs-queue-policies-live`            | Exercise the current Rust service-local keyed queue API on the shared runtime; no process-global state requires isolation. |
| REWRITE | `jobs.terminal-local-job-edges-and-admin-rpcs`   | Exercise terminal `JobRef` behavior and generated Jobs admin RPCs through one native Rust fixture.                         |
| MERGE   | `control-plane.jobs-admin-lists-and-cancels-job` | Covered by `jobs.terminal-local-job-edges-and-admin-rpcs`; control-plane ownership is not a distinct invariant.            |

## Deployment Authority

| Action  | Case                                                                                  | Disposition                                                                                                                           |
| ------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| MERGE   | `authority-plan.preapproved-contract-connects`                                        | Covered with pending-before-approval and post-approval RPC behavior by `service-approval.startup-completes-after-authority-approval`. |
| MERGE   | `authority-plan.presented-update-is-pending-at-connect`                               | The pending assertion becomes the first phase of `authority-plan.presented-update-approved-then-connects`.                            |
| REWRITE | `authority-plan.presented-update-approved-then-connects`                              | Cover pending, rejected, and accepted additive proposals with native participant artifacts and generated Auth RPCs.                   |
| MERGE   | `authority-plan.presented-update-rejected-stays-blocked`                              | Covered by the rejected branch of `authority-plan.presented-update-approved-then-connects`.                                           |
| REWRITE | `authority-plan.incompatible-migration-approved-replaces-contract`                    | Cover rejected and accepted migration branches with native participant and API artifacts.                                             |
| MERGE   | `authority-plan.incompatible-migration-rejected-keeps-old-contract`                   | Covered by the rejected branch of `authority-plan.incompatible-migration-approved-replaces-contract`.                                 |
| KEEP    | `authority-plan.compatible-replacement-auto-allowed-strict`                           | Metadata-only native participant replacement remains a distinct strict-mode invariant.                                                |
| RETIRE  | `authority-plan.mutable-dev-auto-accepts-incompatible-migration`                      | Mutable-dev compatibility was removed from the public deployment contract; strict mode is the sole current behavior.                  |
| RETIRE  | `authority-plan.mutable-dev-rejected-explicit-update-still-blocks`                    | Mutable-dev compatibility was removed from the public deployment contract; strict mode is the sole current behavior.                  |
| REWRITE | `authority-plan.resource-change-migration-approved-and-bound`                         | Prove native resource replacement, removed-handle absence, exact new binding, and desired-state replacement rather than merge.        |
| MERGE   | `control-plane.service-resource-removal-purges-old-binding`                           | Covered by `authority-plan.resource-change-migration-approved-and-bound`.                                                             |
| MERGE   | `control-plane.authority-plan-migration-replaces-desired-state`                       | Covered by `authority-plan.resource-change-migration-approved-and-bound`.                                                             |
| KEEP    | `authority-plan.acceptance-rejects-wrong-classification-expired-and-version-mismatch` | Public generated acceptance validation remains a distinct live invariant.                                                             |

## Service Approval

| Action  | Case                                                                          | Disposition                                                                                                                                         |
| ------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| MERGE   | `service-approval.startup-blocks-before-authority-approval`                   | Covered as the first phase of `service-approval.startup-completes-after-authority-approval`.                                                        |
| REWRITE | `service-approval.startup-completes-after-authority-approval`                 | Prove pending connect, approval/reconciliation, immediate pre-approved connect, and an authorized typed RPC through native artifacts.               |
| MERGE   | `service-approval.approved-service-handles-client-rpc`                        | Covered by `service-approval.startup-completes-after-authority-approval`.                                                                           |
| REWRITE | `service-approval.service-bootstrap-denies-missing-disabled-and-digest-drift` | Preserve fail-closed bootstrap coverage using native participant artifacts, exact digests, and current materialization state rather than manifests. |

## App Approval And Sessions

| Action  | Case                                                                  | Disposition                                                                                                                              |
| ------- | --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| MERGE   | `app-identity-approval.connect-requires-auth-flow`                    | Auth-required callback material is asserted by `auth.local-login-binds-approved-client`.                                                 |
| MERGE   | `app-identity-approval.approved-client-connects`                      | Successful bind and connection are asserted by `auth.local-login-binds-approved-client`.                                                 |
| MERGE   | `app-identity-approval.approved-client-calls-service`                 | The approved typed RPC is asserted by `auth.local-login-binds-approved-client`.                                                          |
| REWRITE | `auth.local-login-binds-approved-client`                              | Cover auth-required callback material, public local login/approval, active admin session state, and an authorized service RPC.           |
| REWRITE | `auth.local-login-rebinds-existing-session-with-updated-authority`    | Cover same-identity rebind and different-identity replacement in one session-key lifecycle.                                              |
| MERGE   | `auth.local-login-replaces-session-when-identity-changes`             | Covered by `auth.local-login-rebinds-existing-session-with-updated-authority`.                                                           |
| REWRITE | `auth.sessions-logout-deletes-session-and-connections`                | Preserve generated logout, durable revocation, connection cleanup, and old-session denial.                                               |
| RETIRE  | `auth.sessions-logout-cleans-connections-after-kick-failure`          | This depended on an old injected TypeScript kick failure; Rust transaction/outbox tests own commit-before-side-effect failure semantics. |
| MERGE   | `auth.sessions-me-reports-app-envelope`                               | Covered by `auth.local-login-binds-approved-client`.                                                                                     |
| REWRITE | `auth.sessions-list-and-connections-list-report-participant-metadata` | Cover app, agent, service, and device list metadata plus current service/device/user envelopes in one native inventory case.             |
| KEEP    | `auth.connections-list-skips-malformed-connection-entries`            | The current test runtime exposes raw presence seeding, and malformed operational KV entries remain a live boundary concern.              |
| MERGE   | `auth.sessions-me-reports-service-envelope-and-current-user-state`    | Covered by `auth.sessions-list-and-connections-list-report-participant-metadata`.                                                        |
| MERGE   | `auth.sessions-me-reports-device-envelope`                            | Covered by `auth.sessions-list-and-connections-list-report-participant-metadata`.                                                        |
| REWRITE | `auth.sessions-me-rejects-stale-user-principals`                      | Cover stale user and stale device principals together through generated `Auth.Sessions.Me`.                                              |
| MERGE   | `auth.sessions-me-rejects-stale-device-principals`                    | Covered by `auth.sessions-me-rejects-stale-user-principals`.                                                                             |
| REWRITE | `auth.session-revoke-denies-reconnect`                                | Cover reconnect denial, runtime presence cleanup, and app, service, and device revocation through generated Auth RPCs.                   |
| MERGE   | `auth.session-revoke-cleans-runtime-connection-presence`              | Covered by `auth.session-revoke-denies-reconnect`.                                                                                       |
| MERGE   | `auth.sessions-revoke-revokes-device-and-service-access`              | Covered by `auth.session-revoke-denies-reconnect`.                                                                                       |
| KEEP    | `auth.grant-overrides-bind-without-user-capability`                   | Portal-selected grant overrides remain a distinct authorization invariant.                                                               |
| REWRITE | `auth.portal-route-selection-and-policy-drive-browser-flow`           | Cover route selection, fallback, built-in protection, selector conflicts, and in-use removal through current portal APIs.                |
| MERGE   | `auth.portal-admin-protects-built-in-and-route-conflicts`             | Covered by `auth.portal-route-selection-and-policy-drive-browser-flow`.                                                                  |
| REWRITE | `auth.capability-groups-and-last-admin-guard-are-enforced`            | Preserve capability-group reference validation and built-in protection; retire removed role-management and last-admin claims.            |
| KEEP    | `auth.users-identities-admin-surfaces-page-and-scope`                 | Pagination, identity unlink, and admin scope remain distinct public Auth behavior.                                                       |
| KEEP    | `auth.account-flow-oauth-callback-runtime`                            | The real HTTP callback and provider binding remain a distinct boundary.                                                                  |

## Device Activation

The old activation-request, wait, and connect-info endpoints remain retired.
Their current product invariants are represented through the existing
proof-bound `/bootstrap/device` route and generated Auth operation/RPC surfaces:

| Action  | Current case                                                               | Disposition                                                                                                   |
| ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| REWRITE | `auth.device-activation-none-connect-revoke`                               | Provision, pending response, confirmation possession, no-review Resolve, ready connect, list, and revocation. |
| REWRITE | `auth.device-activation-required-review`                                   | Required review remains pending until authorized approve/reject; Resolve cannot self-approve.                 |
| REWRITE | `auth.device-activation-rejects-invalid-proof-confirmation-and-deployment` | Reject corrupt/stale proof, wrong confirmation, and mismatched deployment evidence without mutation.          |
| MERGE   | old authority-list and reconnect rows                                      | Covered by the no-review connect/revoke case.                                                                 |
| RETIRE  | old activation-request/wait/connect-info route mechanics                   | Those routes are genuinely deleted and are not compatibility surfaces.                                        |

## TypeScript SQL Outbox

These rows do not belong in the Rust runtime matrix. The still-public TypeScript
service-owned `createSqlOutbox` behavior is now live-owned by
`outbox.typescript-sql-commit-rollback` in `client-test-matrix.json`; one case
covers committed events, committed job submission, rollback silence, multiple
records, and durable dispatched state against the Rust owner.

| Action | Case                                                          |
| ------ | ------------------------------------------------------------- |
| MOVE   | `outbox.commits-event-through-sql-outbox`                     |
| MOVE   | `outbox.dispatches-jobs-through-sql-outbox`                   |
| MOVE   | `outbox.rollback-does-not-publish`                            |
| MOVE   | `outbox.multiple-events-in-one-transaction`                   |
| MERGE  | `outbox.listener-derives-event`                               |
| MERGE  | `outbox.sql-row-state-is-dispatched`                          |
| RETIRE | `outbox.sqlite-010-schema-upgrades`                           |
| RETIRE | `control-plane.outbox-dispatches-after-control-plane-restart` |

## Rust Platform Lifecycle

| Action  | Case                                                                             | Disposition                                                                                                           |
| ------- | -------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| KEEP    | `control-plane.admin-bootstrap-creates-first-local-admin`                        | First-admin bootstrap remains a current public HTTP boundary.                                                         |
| KEEP    | `control-plane.password-reset-change-invalidates-old-password`                   | Password reset/change and old-credential denial remain current public behavior.                                       |
| RETIRE  | `control-plane.http-route-security-requires-admin-session`                       | The claimed `/bootstrap/client` route was removed by the Rust auth clean break.                                       |
| RETIRE  | `control-plane.bootstrap-requires-auth-for-unbound-client`                       | `/bootstrap/client` no longer exists; app authorization now uses account-flow bind and authorization-context refresh. |
| RETIRE  | `control-plane.bootstrap-rejects-unknown-contract-digest`                        | `/bootstrap/client` and combined contract-digest lookup no longer exist.                                              |
| RETIRE  | `control-plane.bootstrap-rejects-non-client-contract`                            | `/bootstrap/client` and combined contract-kind lookup no longer exist.                                                |
| RETIRE  | `control-plane.bootstrap-selects-exact-session-contract-digest`                  | App sessions now bind exact native participant and needs digests through authorization contexts.                      |
| RETIRE  | `control-plane.bootstrap-deletes-session-for-inactive-user`                      | The old bootstrap mutation path was removed; current issuance/context tests own fail-closed user state.               |
| RETIRE  | `control-plane.bootstrap-deletes-session-for-missing-user-projection`            | The old bootstrap projection path was removed.                                                                        |
| RETIRE  | `control-plane.bootstrap-deletes-session-for-insufficient-user-capabilities`     | The old bootstrap capability path was replaced by authority materialization and context issuance.                     |
| RETIRE  | `control-plane.bootstrap-reports-server-time-for-stale-proof`                    | The old client-bootstrap proof format and route were removed.                                                         |
| RETIRE  | `control-plane.bootstrap-rejects-invalid-signature`                              | The old client-bootstrap proof format and route were removed.                                                         |
| RETIRE  | `control-plane.bootstrap-allows-known-inactive-app-digest`                       | The old active-contract distinction was replaced by exact native participant authorization contexts.                  |
| REWRITE | `control-plane.sessions-survive-control-plane-restart`                           | In one restart, prove bound app reconnect, generated State persistence, and service resource binding/data continuity. |
| MERGE   | `control-plane.state-persists-across-control-plane-restart`                      | Covered by `control-plane.sessions-survive-control-plane-restart`.                                                    |
| MERGE   | `control-plane.resources-survive-control-plane-restart`                          | Covered by `control-plane.sessions-survive-control-plane-restart`.                                                    |
| REWRITE | `control-plane.admin-service-deployment-lifecycle`                               | Cover generated create/list/disable/enable/remove, unsafe in-use removal validation, and second-remove rejection.     |
| MERGE   | `control-plane.service-admin-removal-rejects-unsafe-purge-and-noncascade-in-use` | Covered by `control-plane.admin-service-deployment-lifecycle`.                                                        |

## Consolidated Failure Invariants

The historical TypeScript hooks are not restored one-for-one. Their current
externally observable behavior is merged into three live Rust cases. The only
injection surface is target-scoped test SQLite state enabled in the integration
runtime build; it adds no production configuration or generic chaos framework.

| Action  | Current case                                                         | Disposition                                                                   |
| ------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| REWRITE | `auth.validation-failure-persists-no-state-or-actions`               | Invalid mutation leaves state, idempotency, and post-commit counts unchanged. |
| REWRITE | `auth.transaction-failure-rolls-back-state-idempotency-and-actions`  | Repository transaction failure rolls back the complete aggregate.             |
| REWRITE | `auth.post-commit-failure-retries-committed-context-revocation-once` | Committed mutation survives one failed dispatch and converges on retry.       |

The old per-entity fault rows below are merged into those three invariants:

| Action | Case                                                                  |
| ------ | --------------------------------------------------------------------- |
| RETIRE | `control-plane.admin-service-deployment-rollback-fault`               |
| RETIRE | `control-plane.admin-device-deployment-rollback-fault`                |
| RETIRE | `control-plane.admin-service-deployment-validate-before-persist-kick` |
| RETIRE | `control-plane.admin-service-deployment-disable-refresh-rollback`     |
| RETIRE | `control-plane.admin-service-deployment-enable-refresh-rollback`      |
| RETIRE | `control-plane.admin-service-instance-disable-refresh-rollback`       |
| RETIRE | `control-plane.admin-service-instance-enable-refresh-rollback`        |
| RETIRE | `control-plane.admin-service-instance-remove-refresh-rollback`        |
| RETIRE | `control-plane.admin-device-deployment-disable-refresh-rollback`      |
| RETIRE | `control-plane.admin-device-deployment-enable-refresh-rollback`       |
| RETIRE | `control-plane.admin-device-instance-disable-refresh-rollback`        |
| RETIRE | `control-plane.admin-device-instance-enable-refresh-rollback`         |
| RETIRE | `control-plane.admin-device-instance-remove-refresh-rollback`         |
