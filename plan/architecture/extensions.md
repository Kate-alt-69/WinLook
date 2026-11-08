# Extensions, Third-Party Assets and Community Ecosystem

> Status: architecture proposal. Package parsing, runtime compatibility and security validation are separate capabilities.

## Purpose

WinLook should be an open customization platform with a community ecosystem. Users can install local packages and browse independent community catalogues. WinLook does not require every creator to publish through one central WinLook-operated server, and it does not require a WinLook account for local content.

WinLook's own creation and packaging tools belong inside WinLook Manager; there is no separate Creator Studio product in the current scope.

## Content types

The ecosystem may contain:

- WLT Templates and ZIP-based Template Groups;
- WLDE wallpapers, including \`.wl.wp\` and interactive \`.wl.i.wp\` packages;
- Appearance and icon themes;
- desktop widgets;
- Rainmeter-compatible content;
- cursors, sounds and fonts;
- WLWM settings/layout profiles;
- integrations and extensions.

Each content type must declare its format/schema version, required WinLook version, supported OS capabilities, dependencies, assets, permissions, licence/attribution and publisher information.

## Rainmeter compatibility

Rainmeter's official documentation describes \`.rmskin\` as the Rainmeter Skin Installer package format. It is a **modified ZIP format** with additional metadata used to verify package integrity; renaming an ordinary ZIP to \`.rmskin\` is not sufficient. The Skin Packager includes a root skin configuration and may include layouts and plugins. See [Rainmeter — Distributing Skins (.rmskin)](https://docs.rainmeter.net/manual/distributing-skins/) and [Installing Skins](https://docs.rainmeter.net/manual/installing-skins/).

WinLook must separate these stages:

1. Recognize and validate the package container and its integrity metadata.
2. Extract the package safely into a staging location.
3. Parse its skin configuration, resource references and optional components.
4. Determine which features WLDE can emulate and which require a compatible external runtime/plugin.
5. Produce a compatibility report before loading any content.

Reading INI files and extracting assets does **not** provide full Rainmeter runtime compatibility. Full support requires behaviour for measures, meters, variables, actions/bangs, plugins, update cadence, input, layouts and other runtime semantics. Unsupported behaviour must be reported rather than silently ignored.

Rainmeter is GPL-2.0 licensed and its community skins/plugins may carry separate licences. Do not bundle, copy or redistribute Rainmeter code, plugins, skins, fonts or other assets unless the applicable licence permits it. See [Rainmeter source repository and licence](https://github.com/rainmeter/rainmeter).

## Package and runtime trust

Treat all downloaded or user-imported content as untrusted, including packages from a known catalogue.

- Validate archive paths after normalization; reject absolute paths, parent traversal, escaping links, duplicate normalized paths and invalid names.
- Bound archive depth, file count, total expanded size, compression ratio, download size and parser resource use.
- Verify manifest hashes and signatures where available. A hash detects corruption; it does not establish publisher trust.
- Stage extraction and installation; do not execute package contents during import.
- Keep web/interactive wallpaper and widget runtimes sandboxed with explicit filesystem, network, audio and input permissions.
- Native DLLs, scripts and injected modules require stronger warnings and explicit consent. Do not describe untrusted native code as a harmless theme.
- Record permissions and provenance in the installed-content database so users can review and revoke them.
- Provide disable, uninstall, rollback and cache-cleanup paths.

## Community catalogues and Store client

The integrated Store experience is part of WLDE, but catalogues may be hosted independently.

Candidate sources include:
- community-operated HTTP catalogues with a documented schema;
- GitHub repositories/releases;
- self-hosted community sites;
- direct package URLs;
- local files and removable media.

The client should let users add, remove, prioritize and inspect sources. It should not imply that a catalogue is official or trusted merely because it is listed. Each catalogue and publisher needs a trust state and a clear source URL.

Store discovery, ratings, reviews, creator pages, moderation and payments are future product decisions. Local packages must remain usable when a community source is offline.

## Application Integration Bridge

The public integration API should allow an application to opt into WinLook features without requiring every vendor to support WinLook directly.

- Native applications can use documented Rust APIs/SDKs or a stable IPC contract.
- Community adapters can add integrations for applications without native support.
- Independent customization continues to work where supported even if an application has no integration.
- Adapters must declare target identity, supported app versions, required permissions, architecture, install/uninstall behaviour and recovery plan.

Do not assume large application vendors will adopt the API. The ecosystem must remain useful through community-maintained adapters and independent OS-level customization.

## Sources

- [Rainmeter — Distributing Skins (.rmskin)](https://docs.rainmeter.net/manual/distributing-skins/)
- [Rainmeter — Installing Skins](https://docs.rainmeter.net/manual/installing-skins/)
- [Rainmeter source repository and GPL-2.0 licence](https://github.com/rainmeter/rainmeter)
- [GitHub Releases API](https://docs.github.com/en/rest/releases/releases)
