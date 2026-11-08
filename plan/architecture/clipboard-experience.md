# WinLook Clipboard Experience

> Status: planning proposal. This complements Windows Clipboard History; it does not replace, disable or claim ownership of the native Win+V surface.

## 1. Purpose

Add a WinLook-owned, navigation-aware clipboard palette for quickly choosing what to paste, searching recent copied content, and invoking context-appropriate copy/paste actions.

The existing Windows clipboard and native Clipboard History remain available. WinLook must not hijack Win+V or silently replace the system experience.

## 2. Invocation and shortcut policy

The proposed UI is opened through a configurable shortcut and may also be launched from Manager or a WinLook command surface.

Important Windows 11 conflict:
- Microsoft documents Win+Ctrl+V as the shortcut that opens the sound output page of Quick Settings, including output device and volume mixer.
- Therefore Win+Ctrl+V is not WinLook's default clipboard shortcut on Windows 11 and must not be claimed silently.
- Shortcut selection must check the Windows-version conflict table and allow users to select another combination. If the chosen shortcut is reserved, inaccessible or cannot be registered, explain the conflict and leave the OS shortcut working.

References:
- [Microsoft Windows keyboard shortcuts](https://support.microsoft.com/en-gb/accessibility/windows-keyboard-shortcuts)
- [Windows 11 Build 25309 announcement](https://blogs.windows.com/windows-insider/2023/03/02/announcing-windows-11-insider-preview-build-25309/)

The actual default shortcut remains undecided until a conflict audit covers Windows 95 through 11 and common applications.

## 3. Two navigation-aware entry modes

### Paste palette
When invoked in a text editor, browser, file manager or compatible target, show available clipboard entries and actions relevant to the current foreground context.

Candidate content:
- plain text;
- rich text and HTML;
- images;
- file/folder drop lists;
- supported application-specific clipboard formats when safe to preserve.

Actions may include paste as-is, paste plain text, paste without formatting, inspect preview, pin, delete or search. Only expose an action when the selected item's formats support it.

When the user selects an entry, WinLook places the selected data onto the standard Windows clipboard using the preserved supported formats. A separate explicit Paste action may send the normal paste command to the previously focused target after rechecking that the foreground window has not changed. Do not silently type into a different application.

### Copy / capture palette
A second mode may expose contextual capture actions, such as copying selected text through the active application's normal copy command, copying a file selection, or saving an explicitly selected clipboard item into a named collection.

WinLook cannot assume it can read arbitrary selection state from every application. Where the foreground application does not expose a supported selection/automation contract, use the application's normal Copy command or ask the user to select/copy first. Do not inject into arbitrary applications to scrape their selected text.

## 4. UI and navigation

Proposed palette:
- compact search/navigation field;
- Recent, Pinned, Text, Images, Files and Collections views where supported;
- keyboard-first movement and activation;
- type-aware preview and safe metadata;
- per-entry actions in a secondary menu;
- clear indication of the destination action (copy to clipboard vs paste now);
- empty, disabled, unsupported-format and permission states.

The palette should be fast and low-overhead. Do not embed a full browser engine. Keep UI page/components separate under the planned C/UI and rust/UI trees.

## 5. Clipboard storage and native coexistence

WinLook keeps its own optional local index/history. It does not assume that arbitrary WinLook entries can be inserted into, or fully retrieved from, the native Win+V history.

Windows provides clipboard APIs:
- clipboard format listeners (Vista+) notify a window when clipboard content changes;
- clipboard viewer-chain support exists for backward compatibility;
- the WinRT Clipboard history methods such as GetHistoryItemsAsync and SetHistoryItemAsContent are versioned from Windows 10 version 1809, subject to their documented app/thread access constraints.

References:
- [Using the Clipboard (Win32)](https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard)
- [Windows.ApplicationModel.DataTransfer.Clipboard API](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.datatransfer.clipboard)
- [Windows Clipboard support](https://support.microsoft.com/en-US/Windows/Apps/using-the-clipboard)

The architecture must not depend on Windows 10 1809 APIs for every supported OS. Use the relevant OS-specific clipboard adapter and capability result. The older clipboard viewer chain is less robust than clipboard format listeners; this trade-off must be documented and tested.

## 6. Data formats and paste fidelity

Clipboard content is a set of offered formats, not necessarily one simple string. The design must preserve compatible representations when possible:
- Unicode/plain text;
- HTML and rich text;
- bitmap/DIB and image formats;
- file-drop lists and supported storage descriptors;
- application-specific formats only when safe and still meaningful.

Do not convert everything into plain text. Do not assume file clipboard formats contain portable file bytes; they may reference existing paths or use delayed rendering. Validate referenced paths and handle source data becoming unavailable.

Large content may be stored as a bounded external blob with metadata in the index. Preserve original format bytes when possible. Define quotas, retention and deduplication before implementation.

## 7. Privacy and security

- Local-only storage by default; no cloud synchronization in the initial scope.
- User-controlled retention period, maximum storage and clear-history action.
- Per-application exclusion list and a pause/private mode.
- Avoid capturing clipboard updates while secure desktop is active; do not interact with credential UI.
- Do not claim reliable password detection; sensitive-content heuristics are incomplete.
- Never log clipboard contents in diagnostics or crash reports.
- Encrypt or protect local storage if the threat model requires it; document the limits of OS account protection.
- Clipboard content is untrusted input: validate formats and size limits before previewing or inserting.
- Do not execute pasted content, links, scripts or files.

## 8. Profiles and Templates

Profiles may control:
- enable/disable clipboard capture;
- retention and size limits;
- app exclusions;
- preferred paste action;
- shortcut and palette behaviour.

WLT may export these preferences, but never clipboard history entries, clipboard payloads, pinned private content, or machine-specific paths by default.

## 9. Compatibility and failure behaviour

The capability matrix must distinguish clipboard monitoring, history storage, rich-format support, shortcut registration, foreground target validation and native-history integration. A Windows version may support basic clipboard operations while lacking a modern listener or native history API.

If WinLook exits or its shortcut cannot be registered, normal Windows copy/paste and native Clipboard History continue unchanged.

## 10. Acceptance gates

- [ ] Win+V remains the native Windows Clipboard History shortcut.
- [ ] Win+Ctrl+V remains the Windows 11 sound Quick Settings shortcut.
- [ ] No default shortcut is published until per-OS conflict audit passes.
- [ ] The history is additive and local-only by default.
- [ ] Rich formats and file-drop data are not silently flattened.
- [ ] Paste action revalidates its foreground target.
- [ ] Private mode, app exclusions, retention and clear-history work.
- [ ] Clipboard data never enters logs or WLT exports.
- [ ] Each OS adapter has monitoring and fallback tests.

## 11. Open decisions

- Default shortcut by Windows generation.
- Whether WinLook reads the native Windows 10/11 history where permitted, or maintains a completely separate history in the first release.
- Exact paste action semantics and target-focus safety.
- Collection/favorites model and storage quotas.
- Supported clipboard formats by OS.
- How copy/capture mode obtains selected content from applications without integration.
