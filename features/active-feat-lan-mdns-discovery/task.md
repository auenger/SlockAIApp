# Tasks: feat-lan-mdns-discovery

## Task Breakdown

- [x] T1: Add `mdns-sd` crate dependency and create `discovery.rs` module in `src-tauri/src/runtime/a2a/`
- [x] T2: Implement mDNS service registration (publish `_a2a._tcp.local.` when LAN server is running)
- [x] T3: Implement mDNS service browsing/discovery (find other AgentsZone instances on LAN)
- [x] T4: Add Tauri IPC commands for discovery lifecycle (start/stop/get_peers)
- [x] T5: Add TypeScript types and IPC functions in frontend
- [x] T6: Create `useLanDiscovery` React hook
- [x] T7: Create `LanPeersPanel` UI component in Settings
- [x] T8: Integrate LanPeersPanel into Settings page

## Progress Log
| Date | Progress | Notes |
|------|----------|-------|
| 2026-04-17 | T1-T8 complete | All 8 tasks implemented. Rust backend (discovery.rs + IPC commands), TypeScript types + IPC, React hook, UI panel. 5 unit tests pass. |
