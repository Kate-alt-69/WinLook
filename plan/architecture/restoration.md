# Restoration, Recovery and Clean Uninstallation

## Principle
Uninstalling WinLook must reverse WinLook-owned changes wherever it can safely identify and restore the original state. Deleting the program directory is not a clean uninstall.

## Change journal
Before each mutation, record component/setting ID, original value/scope, value applied by WinLook, timestamp/version, restore procedure, whether an external later change is detected and transaction status. Do not store credentials or unrelated user data.

## Apply transaction
1. Detect capability and read current state.
2. Snapshot original value.
3. Validate requested modification/permission.
4. Apply in a staged/reversible manner.
5. Verify result.
6. Commit journal entry.
7. On failure, restore prior state and report incomplete recovery.

## Clean uninstallation
1. Stop future WLWM targeting/injection.
2. Notify in-process modules to disable/unload when safe.
3. Stop WLDE playback/workers.
4. Restore desktop icon presentation and Local Group member icon visibility.
5. Remove WLOSDE-owned surfaces.
6. Restore Appearance/Icon Engine changes and taskbar/Start Menu state changed by WinLook.
7. Restore fonts/cursors/sounds installed by WinLook according to ownership and user choice.
8. Restore display settings changed by WinLook if the original device/config remains identifiable.
9. Restore wallpaper/desktop state.
10. Remove WinLook-owned service/tasks/startup entries.
11. Remove WinLook-owned files/caches only after restoration.
12. Preserve user's original files, shortcuts and applications.
13. Produce a recovery report for items not restored.

## External changes
If the current value differs from the value last applied by WinLook, assume the user/another app may have changed it. Do not blindly overwrite it with an old snapshot.

## Crash recovery
- Persist restoration metadata before changing desktop state.
- WLDE/BGW bridge crash: release visual suppression and show original native desktop items; if an in-process bridge is still loaded, it must detect controller loss and enter the native-visible safe state.
- Explorer restart: BGW rediscovers the desktop-owning process, revalidates the per-version adapter and reattaches only after the prior restore transaction is reconciled.
- WLWM crash: stop new injection and return to native behaviour where safely possible.
- BGW crash: injected modules detect controller loss and enter a defined safe state.
- If unload isn't safe, prevent future targeting and provide a reboot-assisted cleanup path.
- Provide safe recovery mode that can disable experimental integrations without normal Manager startup.

## Acceptance tests
Crash WLDE while Groups active; crash BGW with modules loaded; uninstall while target apps open; interrupt theme apply; user changes settings externally; monitor disconnect before restoration; interrupted uninstall and reboot; verify original shortcuts/files remain unchanged.

## Detailed journal and transaction model

### Proposed journal entry

Every WinLook-owned change should record transaction ID, ordered operation index, component and adapter ID/version, setting/resource identifier, machine/user/profile scope, previous value and ownership state, requested value, value successfully applied, observed post-apply value, timestamp, operation status, restoration method/capability, external-change state, backup reference/hash and error/recovery reason.

Sensitive values must not be copied into the journal. Use references to protected backups where needed and permitted.

### Transaction states

Proposed states: Prepared, SnapshotSaved, Applying, Applied, Verified, Committed, RollingBack, Restored, PartialRestore, Conflict and Failed.

A transaction is not committed merely because an API call returned success. Verify observable state before marking it Verified/Committed.

### External-change conflict rule

Before restoring:
- If current value still matches the last value WinLook recorded as applied, restore the saved previous value.
- If current value differs, treat it as an external change and do not overwrite blindly.
- Offer a user choice to keep the external value, reapply WinLook's value, or restore the earlier snapshot when safe and understood.
- Record the decision and result.

This prevents WinLook from undoing a change the user made after applying a theme.

### Ordering and recovery

- Snapshot required state before mutation.
- Apply one reversible operation at a time in explicit dependency order.
- Persist progress at boundaries where a crash could leave ambiguity.
- Roll back completed operations in reverse dependency order.
- Make recovery idempotent; rerunning it must not compound a change.
- If rollback is incomplete, enter PartialRestore and provide a manual report.

### Uninstall contract

The uninstaller removes WinLook-owned services, startup entries, files and configuration from an explicit inventory. It restores settings only when ownership and external-change checks allow it. It preserves user templates, downloaded assets and logs when the user chooses to retain them. It never deletes unrelated shortcuts, original desktop items or user files.

### Irreversible or conditional operations

An operation without a reliable restoration path is not eligible for normal automatic application. Redesign it, label it experimental with explicit approval and independent recovery, or exclude it.

### Acceptance gates

- [ ] Journal schema is versioned and crash-safe.
- [ ] Every adapter declares snapshot, verify and restore behaviour.
- [ ] External user changes are detected and preserved.
- [ ] Fault injection at each transaction boundary is recoverable.
- [ ] Partial restore has a visible, actionable report.
- [ ] Clean uninstall is tested after normal use and simulated crashes.

## WLDE Explorer Desktop Bridge write-ahead journal contract

**Status: required architecture gate; not yet implemented.** This contract is
for future native Desktop Bridge changes. The existing
`DesktopStateStore` is configuration persistence only and does not satisfy
any of these requirements. Until this journal and an adapter-specific
recovery proof exist, keep native icon suppression, Shell registration
changes, hook/presenter activation and Group presentation disabled.

### Journal ownership and scope

- BGW is the single writer for the interactive user's Desktop Bridge journal.
  Manager communicates through a future authenticated BGW IPC surface; it
  never edits the journal file directly.
- The journal is separate from `desktop-state.v1.bin`. It stores only
  typed restoration metadata: operation/transaction IDs, monotonic sequence,
  adapter ID and version, allowlisted resource ID, user/session scope,
  before/desired values, state and bounded adapter ownership evidence.
- Never persist executable commands, arbitrary registry/file paths, pointers,
  HICON/HBITMAP values, pixel buffers, Shell item contents, credentials or
  arbitrary code. A resource ID must resolve to a compiled, allowlisted
  adapter implementation. Adapter-specific payloads use strict versioned
  codecs and fixed size/count limits.
- Create and validate the per-user journal and its security descriptor before
  permitting any native mutation. If the journal cannot be opened, validated,
  appended or flushed, fail closed and preserve native desktop presentation.
  A missing/corrupt journal while a prior mutation may still be active is a
  recovery failure, not an empty journal.
- The first implementation should use an append-only log on a supported local
  filesystem, with bounded record and total-file sizes. Do not compact or
  discard records while any operation is Committed, Applying, Restoring,
  Conflict or otherwise unresolved. A later compactor needs its own crash
  protocol and acceptance tests.

### Write-ahead transaction protocol

One journal operation represents one reversible native mutation. Never group
multiple independent Shell mutations behind a single unverified commit.

1. Discover the exact adapter/build/architecture and current desktop host.
   Confirm that this adapter declares capture, apply, inspect, ownership proof,
   verify and restore operations for the target resource.
2. Read the original state and capture an adapter-specific ownership/recovery
   token. Validate the requested state and ensure the adapter can distinguish
   its own mutation from an external change.
3. Append a complete `Prepared` record containing the original value, desired
   value, adapter/resource versions, scope and ownership evidence. Flush the
   record successfully before proceeding.
4. Append an `Applying` transition and flush it successfully. The adapter must
   not be called before this write-ahead boundary has completed.
5. Apply exactly one reversible mutation. Never do network, filesystem
   enumeration or unbounded work inside an Explorer callback.
6. Re-read the resource through the adapter. Only if the observed state equals
   the desired value **and** the adapter confirms ownership may BGW append and
   flush `Committed`. A successful API return alone is not verification.
7. On an apply error or ambiguous readback, do not assume the operation failed
   cleanly. Leave the durable intent record available for recovery and inspect
   the real resource before deciding whether to roll back or report Conflict.

Before restoration, append and flush `Restoring`. Re-inspect the resource
immediately before applying the restore. Restore the captured original value
only when the observed value still matches the recorded WinLook-applied value
and adapter ownership proof still matches. Re-read after restoration, then
append and flush `Restored`. Every transition must be idempotent.

### Recovery interpretation

- Latest state `Prepared`: the protocol guarantees no native call has begun.
  Discard/close the unstarted operation without applying or restoring anything.
- Latest state `Applying`: the mutation may or may not have happened. The
  adapter must inspect current state. If it can prove the original state is
  still present, record a safe no-op outcome. If it can prove the requested
  state is present and belongs to this operation, restore and verify it.
  Otherwise record `Conflict`; never infer ownership from value equality
  alone.
- Latest state `Committed`: treat the mutation as active. Reattach only when
  the exact adapter version and its ownership evidence remain valid. On
  disable, shutdown or uninstall, restore only after a fresh compare-and-restore
  check.
- Latest state `Restoring`: inspect whether the original state is already
  present. If so, complete the `Restored` record. If the applied state remains
  and ownership is proven, retry the idempotent restore. Any third state or
  lost ownership becomes `Conflict`.
- `Restored`, `Conflict` and explicitly abandoned outcomes are terminal for
  automatic recovery. A conflict requires an actionable report and an explicit
  resolution; recovery must not silently overwrite a user's later change.
- Restore operations in reverse dependency order. Recovery must be safe to run
  repeatedly after another crash.

A value comparison is necessary but may not prove who changed a Shell resource.
An adapter that cannot provide reliable ownership evidence, detect external
changes, or restore after controller loss is not eligible for automatic
mutation. It must leave the native representation visible and report the
unsupported capability.

### Windows journal durability contract

The journal writer should use a single append handle and append one complete,
length-delimited, checksummed record per transition. On Windows, the native
backend must flush the writable journal handle using `FlushFileBuffers`
before the associated side effect is allowed. Treat any write/flush error as
fatal to that operation. Flush only at transaction boundaries, not per frame;
this is control-plane work, not a rendering hot path.

Do not claim that a temp-file rename alone is a durable write-ahead commit.
Microsoft documents `FlushFileBuffers` as flushing buffered information for
the specified file, and warns about the cost of frequent flushes. The
`MOVEFILE_WRITE_THROUGH` flag documents behavior for file moves (including
copy/delete moves); it is not a substitute for flushing each journal record.
`ReplaceFileW` documents `REPLACEFILE_WRITE_THROUGH` as unsupported, so that
flag must not be used as a journal durability guarantee. These constraints are
recorded as WLDE-037–039 in the source ledger.

A valid complete record must pass magic/version/length/sequence/state/adapter
validation and checksum verification. A short trailing fragment may be
recognized as an incomplete final append only because the protocol forbids
the native call until the full corresponding transition has flushed. A
complete record with a bad checksum, invalid transition, duplicate sequence,
unknown adapter/resource, missing journal, or untrusted ACL is corruption or
tampering: stop further native changes and report recovery required. CRC32
detects accidental damage; it is not authentication. File ACLs and allowlisted
typed decoding are mandatory.

This contract does not assume that every storage device, redirector or
filesystem has identical power-loss behavior. Supported local filesystems and
devices must pass process-termination, OS-crash and power-loss-oriented fault
testing. Unsupported/remote journal locations fail closed.

### Required fault-injection matrix

- Crash before Prepared, during append, after append but before flush, after
  Prepared flush, during Applying append, after Applying flush, during native
  apply, after native apply but before readback, after readback, during
  Committed append/flush, during Restoring, after restore but before readback,
  and during Restored append/flush.
- Truncate every possible byte range of the final record; corrupt each header,
  payload, sequence and checksum field; replay, duplicate and reorder records.
- Simulate external changes to the resource before apply, between inspect and
  apply, after commit and during restore. Verify the adapter never overwrites
  an unowned external change.
- Crash/restart BGW and Explorer independently; restart the host with a
  different adapter version, process identity, OS build or architecture.
- Deny journal create/write/flush access, fill the volume, remove the journal,
  change its ACL and supply an unsupported filesystem. Every case must prevent
  new mutations and produce a recovery status.
- Verify restoration is idempotent, reverse ordered and preserves original
  desktop items, shortcuts, file contents and user-owned settings.

### Acceptance gate

- [ ] Versioned append-only journal and bounded typed record codec exist.
- [ ] Journal ACL/owner and supported storage are checked before native work.
- [ ] Every adapter implements capture, apply, inspect, ownership proof,
      verify and restore, with build/version scoping.
- [ ] The durable Prepared and Applying boundaries precede native mutation.
- [ ] Recovery handles every state above and detects external changes.
- [ ] Fault injection covers every persistence/native-call boundary.
- [ ] BGW/adapter controller loss restores native-visible behaviour without
      requiring Manager startup.
- [ ] Disable, Explorer restart, BGW crash, OS restart and uninstall all pass
      on each supported Windows generation.
- [ ] Only after these gates pass may the corresponding presenter or native
      Shell mutation be enabled.
