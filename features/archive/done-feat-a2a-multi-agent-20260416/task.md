# Tasks: feat-a2a-multi-agent

## Task Breakdown

### 1. Push Notification Receiver (`push.rs`)
- [x] Push notification event types (TaskCompleted, TaskFailed, InputRequired, etc.)
- [x] PushNotification struct with idempotency support (event_id dedup)
- [x] PushCallbackConfig management (register / unregister / list configs)
- [x] PushNotificationManager with processed_events set and auto-eviction
- [x] URL validation (SSRF prevention: only localhost + private networks)
- [x] HMAC-SHA256 signature verification (inline implementation)
- [x] Tauri event emission (a2a://task-updated, a2a://task-completed, etc.)
- [x] Unit tests (event types, config CRUD, URL validation, HMAC, idempotency)

### 2. Task Delegation Engine (`delegation.rs`)
- [x] `DelegationRequest` struct (from_agent, to_agent, task_description, context_summary, parent_task_id)
- [x] `DelegationStatus` state machine (PENDING → SENT → ACKNOWLEDGED → IN_PROGRESS → COMPLETED/FAILED)
- [x] `DelegationManager` with CRUD operations
- [x] Status update lifecycle (create → update_status → set_result / set_error)
- [x] Cancel and retry support
- [x] `build_delegation_message()` — formats context for target agent
- [x] `extract_context_summary()` — extracts recent messages as compact summary
- [x] List/filter by agent (from_agent, to_agent, active_only)
- [x] Unit tests (status lifecycle, cancel, retry, list filtering, message building)

### 3. Cross-Agent Artifact Store (`artifact_store.rs`)
- [x] `ArtifactRef` struct (id, producer_agent_id, name, file_path, content_hash, mime_type)
- [x] `ArtifactRecord` with content parts and consumer tracking
- [x] `ArtifactStore` with in-memory registry + filesystem storage
- [x] Register artifacts from file path or inline content
- [x] Query: list_all, list_by_producer, list_by_task, search
- [x] Get content, record consumption (idempotent)
- [x] Delete artifacts
- [x] FNV-based content hash for integrity verification
- [x] Unit tests (CRUD, consumption, search, grouping, delete)

### 4. @mention Trigger Upgrade
- [x] Existing `mention.rs` already supports @mention parsing
- [x] Existing `a2a_trigger.rs` already handles A2A chain execution
- [x] Delegation engine integrates with existing Channel @mention flow
- [x] ConnectionMode-aware (local vs remote) via existing AgentManager

### 5. IPC Commands (`collaboration.rs`)
- [x] `CollaborationState` managed state (DelegationManager + ArtifactStore + PushNotificationManager)
- [x] `collaboration_delegate` — create delegation with ConnectionMode resolution
- [x] `collaboration_list_delegations` — list with optional agent_id filter
- [x] `collaboration_cancel_delegation` — cancel active delegation
- [x] `collaboration_retry_delegation` — retry failed/timed-out delegation
- [x] `collaboration_list_artifacts` — list with agent/task filters
- [x] `collaboration_get_artifact` — get content with consumption tracking
- [x] `collaboration_search_artifacts` — search by name
- [x] `collaboration_register_artifact` — register new artifact
- [x] `collaboration_register_push_url` — register push notification endpoint
- [x] `collaboration_list_push_configs` — list push configs
- [x] `collaboration_unregister_push_url` — unregister push config
- [x] `collaboration_process_push_event` — manually process push event
- [x] All commands registered in `lib.rs` invoke_handler

### 6. Frontend: Collaboration UI
- [x] `CollaborationView.tsx` — tabbed view (Delegations / Artifacts / Events)
- [x] `AgentTaskCard.tsx` — delegation status card with actions
- [x] `PushEventToast.tsx` — auto-dismissing notification toast
- [x] Agent-grouped artifact browser
- [x] Push notification config management UI
- [x] Events timeline with type badges

### 7. Frontend: State & Hooks
- [x] `useCollaboration` hook — delegation management
- [x] `usePushEvents` hook — subscribe to push notification events
- [x] `useArtifacts` hook — artifact querying and management
- [x] TypeScript types in `types.ts` (DelegationInfo, ArtifactInfo, PushEventPayload, etc.)
- [x] IPC wrappers in `ipc.ts` for all collaboration commands

### 8. Integration & Testing
- [x] 35 Rust unit tests passing (push: 12, delegation: 11, artifact_store: 7, + misc)
- [x] TypeScript type checking passes (tsc --noEmit clean)
- [x] cargo build succeeds
- [ ] End-to-end delegation flow test (requires running app)
- [ ] Push notification round-trip test (requires running app)

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-14 | Feature created | Initial task breakdown |
| 2026-04-16 | Implementation complete | All 8 task groups implemented, 35 tests passing |
