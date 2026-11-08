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
- WLDE crash: release visual suppression and show original native desktop items.
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

