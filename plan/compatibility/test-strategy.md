# WinLook Compatibility and Verification Strategy

> Status: test methodology proposal. A host build, source citation or VM boot is not a universal Windows support claim.

## 1. Evidence vocabulary

| Label | Meaning | Permitted claim |
|---|---|---|
| Documented | Primary source documents a platform/API fact | State only the documented scope |
| Source-reviewed | Relevant upstream header/source inspected | Describe source observation, not runtime support |
| Host-tested | Portable component passed on named host/compiler | Claim only that result |
| VM-tested | Feature passed in VM with exact OS/config recorded | Claim VM-tested on that setup |
| Hardware-tested | Passed on physical machine with hardware/driver recorded | Claim only that setup |
| Partially verified | Some preconditions passed, relevant paths remain untested | State gaps |
| Research required | Evidence incomplete or conflicting | Do not call supported |
| Unsupported | Requirement unavailable or intentionally excluded | Explain why |

Documentation and runtime evidence are distinct. A Microsoft minimum-client statement does not prove the WinLook adapter; a C89 host build does not prove a Windows 95 binary.

## 2. Proposed support tiers

Exact OS membership and release promises remain a Phase 0 decision.

- **Tier A — OS verified:** installer, core Manager/Agent, restoration and listed feature set pass the exact OS test record.
- **Tier B — limited verified:** product launches with a deliberately smaller documented feature set; unavailable modules remain disabled.
- **Tier C — experimental:** narrow configuration, explicit opt-in and clear recovery route.
- **Research only:** design/prototypes exist but no product support is claimed.
- **Excluded:** operation deliberately not attempted.

Do not assign a whole Windows generation a tier based on one API or one VM.

## 3. Required environment record

Record exact product/version/edition/service pack/build/revision; OS and process architecture/bitness; CPU family/instruction baseline; RAM/GPU/VRAM/driver; VM or physical hardware; compiler/linker/SDK/CRT/flags; imported DLLs and redistributables; display topology/resolution/refresh/DPI; session and privilege; feature/module revision; configuration; preconditions; steps; expected/observed results; logs/screenshots/trace paths with sensitive data removed; date; reviewer; limitations.

## 4. Test layers

### A. Source and static validation
- API minimums and imported DLL audit.
- Public contracts and dependency direction.
- Manifest/schema migrations.
- Unsafe paths, extraction and privilege expansion.
- Documentation links and compatibility claims.

### B. Build and unit tests
- Compile every language/toolchain target independently.
- Test parsers, serializers, hashing, path normalization, cache invalidation and state machines.
- Test overflow, invalid/truncated/corrupt data and unsupported versions.
- Separate portable host tests from OS-specific tests.

### C. Adapter tests
- API presence/absence and late-binding paths.
- Device/display changes, session notifications and process termination.
- Available, partial, unsupported and unknown capability results.
- Missing optional API must not prevent startup.

### D. VM and hardware
- Clean snapshots for install, update, recovery and uninstall.
- Low-resource configurations and older GPU/driver combinations.
- Label VM evidence separately from physical hardware.
- Re-test hardware-sensitive features on physical devices before broad claims.

### E. Fault injection
Force failure at each transition: process start/stop/crash; IPC timeout/malformed message/protocol mismatch; cache corruption/stale identity; module load/handshake/unload failure; renderer/device loss; monitor disconnect; template extraction/import/apply rollback; interrupted update/uninstall; journal write interruption/recovery.

### F. Security and privacy
Test traversal, archive bombs, symlink/reparse escape, duplicate normalized names, malformed manifests, oversized values, unauthorized IPC, replay, privilege boundary, secret/log redaction, extension isolation and denied capabilities. Confirm WinLook never accesses credentials or secure desktop.

### G. Accessibility and performance
Test keyboard-only operation, focus order, screen reader labels, high contrast, scaling and reduced motion. Measure idle CPU/GPU/RAM/VRAM, frame/decode time, wakeups, GDI/USER handles on legacy paths, pause/resume, Game Mode and sustained resource pressure.

## 5. Feature acceptance record

Every feature record contains:
1. Feature ID and owner.
2. Supported operation and explicit non-goals.
3. Minimum OS/API/runtime/driver prerequisites.
4. Capability probe and reason codes.
5. Test environments and exact results.
6. Failure/fallback behaviour.
7. Restoration and disable procedure.
8. Resource budget.
9. Accessibility behaviour.
10. Evidence links and reviewer.
11. Current evidence label and next evidence required.

## 6. Compatibility matrix rules

- Record each Windows release separately; do not merge 95/98/Me or NT4/2000/XP into one runtime claim.
- Separate OS capability from WinLook adapter readiness.
- Separate optional redistributables from a clean-install baseline.
- Track OS architecture separately from target application bitness.
- Track API minimum, actual presence and tested WinLook behaviour as different fields.
- Keep unknown values visible.
- Update per-OS page, matrix and source ledger together after a compatibility decision.

## 7. Release-blocking failures

Block a release for an unresolved issue that can prevent Windows/target applications from starting; corrupt or permanently hide native desktop items; leave display settings unusable without recovery; overwrite user changes during restore; cross privilege boundaries without consent; execute untrusted content outside its sandbox; strand the user at session/security transitions; or create restart loops/unbounded resource use.
