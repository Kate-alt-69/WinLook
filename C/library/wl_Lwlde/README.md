# wl_Lwlde

C89 companion to the portable WinLook Desktop Engine scene core.

## Scope

- Caller-owned, bounded monitor topology storage (maximum 16 monitors).
- Geometry validation and Fill/Fit/Stretch/Center source-to-monitor placement.
- One scene-owned wl_Lanimate frame clock shared by all active tracks.
- Visibility-aware frame requests; Unknown stays eligible, fully occluded scenes
  hold their last frame, static scenes do not poll a frame clock.
- Normal 60 FPS, Optimize/Power Saver 30 FPS and Suspend policies.
- Non-owning references to existing wl_Lcpuv and wl_Lgpuv governors.
- Capability checks that keep interactive content and Groups unavailable until
  a platform adapter explicitly proves support and crash-safe restoration.
- Immutable provider metadata for Wallpaper Engine Workshop plus free/open
  catalogues (Openverse, Wikimedia Commons and NASA). The C89 API exposes
  provider descriptions, URLs, access/pricing class and item-level rights notes;
  platform UI handles browser launching and file picking.

No allocation occurs in the topology storage, placement, condition update or
scene tick paths. The caller supplies monitor storage and serializes governor
access as required by CPUv/GPUv. The archive links against wl_Lanimate,
wl_Lcpuv and wl_Lgpuv; the root CMake aggregator wires those targets.

## Not implemented here

The Store metadata API is intentionally a provider directory, not a catalogue
 search/download client. It never bypasses Steam subscriptions or redistributes
 creator packages, and it does not assume that provider-level `Free` status
 overrides an individual asset's license.

This library does not create a desktop window, draw pixels, decode media,
inspect Explorer, enumerate real monitors, hide native desktop icons or run
untrusted widget code. It is a C89 source API; that alone does not prove a
binary can run on Windows 95 or any other legacy target.

Run the paired C tests from the repository root with
python build.py verify c --sanitizers, or use CMake through
C/library/CMakeLists.txt.
