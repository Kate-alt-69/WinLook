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
