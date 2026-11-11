# WinLook Vector Animation and SVG Contract

> Status: design proposal, 2026-11-09. This document defines the intended beta contract; it does not claim the current beta-labelled crate is complete or that a renderer exists.
>
> Target baseline: Windows 10 for the first Rust and C implementations.

## 1. Purpose

The animation libraries must be useful for more than moving a window or fading a control. They should let WinLook engines and third-party developers describe, transform, style, animate and render vector objects while keeping the animation model independent from the graphics API.

The intended flow is:

1. A caller builds or imports a vector document.
2. The document owns a view box, a hierarchy of groups and vector items, geometry, transforms and paint/style data.
3. Animation tracks target named or strongly typed properties on those objects.
4. A timeline samples the properties at a caller-supplied monotonic time.
5. A renderer consumes the resulting immutable scene snapshot or render command list.
6. CPU/GPU work is admitted through CPUv/GPUv before the corresponding WinLook task or GPU resource/submission is scheduled.

The Rust `wl_animate` and C `wl_Lanimate` implementations must share observable behavior and test vectors. They do not need identical internal layouts.

## 2. Design layers

Keep five layers separate:

| Layer | Owns | Must not own |
|---|---|---|
| Values and math | vectors, colors, rectangles, matrices, interpolation, easing | wall-clock timers, HWNDs, device objects |
| Vector scene | path geometry, groups, items, styles, IDs and document bounds | GPU resources or OS window state |
| Animation model | tweens, property tracks, timelines, sequences, parallel groups and events | worker-thread scheduling or graphics submission |
| SVG import/export | parsing, normalization, capability report and safe serialization | scripts, arbitrary external resource fetching or executing content |
| Rendering adapters | CPU rasterization, Direct2D/Direct3D and Windows composition integration | changing the logical animation contract |

This split allows tests to run without a display, graphics driver or Windows session.

## 3. Vector and geometry types

### Required value types

The public API should cover:

- Scalar: `f32`, `f64`, integer/normalized channels where conversion is explicit.
- Vectors: `Vector2`, `Vector3`, `Vector4`; constructors, dot product, length, normalization where defined, addition/subtraction/scalar operations and interpolation.
- Geometry: `Point2`, `Size2`, `Rect`, `Insets`, line segment and bounding box.
- Transforms: `Matrix3x2` (2D affine), a typed/decomposed `Transform2D`, and a documented `Matrix4x4`/3D transform type only when a 3D renderer requires it.
- Curves: quadratic and cubic Bézier segments, elliptical arcs, path contours and closed/open state.
- Scene objects: document, group/layer, path, rectangle, rounded rectangle, circle, ellipse, line, polygon and polyline.

Matrix coefficient order, multiplication order, coordinate origin, units, angle units and transform composition must be documented and tested. The API must never rely on a caller guessing whether `A * B` applies A first or B first.

### Path representation

Normalize parsed SVG commands into a documented internal representation while preserving:
- multiple subpaths and each subpath's start/current point;
- open versus closed contours;
- line, cubic Bézier, quadratic Bézier and elliptical arc geometry;
- zero-length segments where they affect marker, stroke or path semantics;
- original command/source offsets for useful parse diagnostics.

Support all SVG path-data command families: `M/m`, `Z/z`, `L/l`, `H/h`, `V/v`, `C/c`, `S/s`, `Q/q`, `T/t` and `A/a`. Repeated argument groups, implicit line groups after moveto, relative coordinates, decimal/exponent numbers, comma/whitespace separators, compact signed values and compact arc flags must follow the SVG grammar. The parser must reject malformed or non-finite input with byte offsets; it must not silently reinterpret malformed commands as valid geometry.

A path morph is allowed only when the two paths have compatible normalized topology (same subpath/segment structure and compatible command kinds). Otherwise return a specific incompatibility result and let the caller choose cross-fade, replace, or a separately generated resampling/morph strategy. Never zip arbitrary command arrays and claim that the result is a valid morph.

Reference: [SVG 2 Paths](https://www.w3.org/TR/SVG/paths.html).

## 4. Vector document and item model

Proposed public model:

- `VectorDocument`: view box, optional intrinsic size, ordered root children, document metadata and import report.
- `VectorGroup`: stable ID/name, ordered child items, local transform, visibility and group opacity.
- `VectorItem`: stable ID/name, geometry, local transform, style and optional accessibility label.
- `VectorPath`: normalized path segments, bounds/cache generation and path-level hit-test geometry.
- `VectorStyle`: fill, stroke, stroke width/cap/join, fill rule, dash pattern/offset, overall opacity, fill opacity and stroke opacity.
- `Paint`: solid color, linear gradient and radial gradient. A paint server is a referenced immutable resource, not a copied GPU handle.
- Gradient: coordinate system, transform, spread method, ordered stops, stop color and stop opacity.
- `SceneSnapshot`: immutable renderer-facing state for one sampled time; renderers do not mutate the source animation model.

Stable IDs and explicit ownership matter: animations must keep targeting the same item when a document is rebuilt or a renderer loses/recreates its device. Resource caches are keyed by logical geometry/style version plus adapter/device identity, not a stale raw device pointer.

## 5. Color, alpha and transparency

Alpha is a first-class part of the API, not a special-case fade effect.

The beta contract should distinguish:
- source color encoding (for example sRGB input versus linear-light values);
- RGB channels from the alpha channel;
- object/group opacity from fill opacity, stroke opacity and gradient-stop opacity;
- straight-alpha input/storage from premultiplied-alpha compositing;
- color interpolation from final blend/composition.

Recommended pipeline:
1. Accept explicit color types and conversions; do not silently treat sRGB values as linear.
2. Preserve alpha separately and normalize opacity to 0..=1.
3. Convert to linear-light, premultiplied-alpha values at the renderer/compositing boundary where the backend requires it.
4. Blend premultiplied colors to avoid dark fringes around transparent edges.
5. Handle zero-alpha unpremultiplication without division by zero or NaN.
6. Keep group opacity semantics distinct: applying group opacity means compositing the group as a whole, not multiplying every child independently when overlap would change the result.

Required types include a documented RGBA representation, an explicit alpha mode, normalized `Opacity`, and explicit sRGB/linear conversion helpers. Legacy callers may retain a simple `Rgba` convenience type, but its channel range and color space must be stated.

SVG paint semantics include `opacity`, `fill-opacity`, `stroke-opacity` and `stop-opacity`; these cannot be collapsed into one value without changing output. See the [SVG 2 rendering model](https://www.w3.org/TR/SVG/render.html).

## 6. SVG support profile

“Supports SVG” must mean a named, testable profile. A path-data parser is not an SVG document importer, and a subset renderer must report omitted features rather than pretend they were rendered.

### Core SVG profile (beta target)

Support the common exported-icon/illustration subset:

- Root `svg`: `viewBox`, width/height, coordinate mapping and `preserveAspectRatio`.
- Shapes: `path`, `rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`.
- Hierarchy: `g`, nested transforms, document order/z-order and group opacity.
- Reuse: `defs` and internal-fragment `use` references with cycle/depth limits.
- Transforms: translate, scale, rotate, skew and matrix, following SVG transform-list composition.
- Paint: solid fill/stroke, fill rule, stroke width/cap/join/dashes, linear/radial gradients, gradient stops and stop opacity.
- Appearance: fill/stroke/overall opacity, visibility/display, color inheritance and a deliberately bounded subset of presentation attributes.
- Units: user units and viewBox mapping first; physical/font-relative units only where a documented conversion context exists.

### Extended profile (post beta.1)

Evaluate, implement and test in separate feature gates:
- clipping paths and clip rules;
- masks and filter primitives;
- pattern paint servers;
- markers;
- text, font selection and text-on-path;
- embedded raster images with strict size/decode budgets;
- CSS style blocks, selectors and cascade rules;
- additional SVG 2 presentation properties.

### Explicitly excluded from untrusted SVG by default

- script elements and event-handler attributes;
- `foreignObject` and embedded active content;
- arbitrary external file/network references;
- external fonts or linked resources without explicit caller-provided resolver/consent;
- unbounded XML entity expansion, recursive `use`, excessive nodes, huge path data or decompression bombs.

The importer must expose a configurable but safe limit set: document bytes, node count, nesting depth, path command count, referenced-resource count, image dimensions/decoded bytes and total geometry budget. Parse failures are errors. Unsupported-but-well-formed elements/properties appear in an `SvgImportReport` with a stable code, location and severity. The caller chooses strict rejection or partial import; the library never silently claims full fidelity.

Microsoft Direct2D documents SVG support beginning with Windows 10 Creators Update but supports only a defined subset of SVG features. Therefore the WinLook document model and import report must remain independent from the specific Windows renderer's subset: [Direct2D SVG support](https://learn.microsoft.com/en-us/windows/win32/direct2d/svg-support).

## 7. Animation API completeness

The beta API must define more than a scalar tween.

### Property animation

- typed interpolation and easing;
- keyframe tracks for any supported animatable property;
- delay, duration, playback speed and seeking;
- pause, resume, cancel, restart and explicit disposal;
- repeat count, infinite repeat (opt-in), autoreverse/yoyo and repeat delay;
- deterministic boundary behavior at exact keyframe/start/end timestamps;
- explicit fill behavior before start and after finish;
- conflict/interruption policy when two animations target the same property.

### Composition

- sequence: run children in a defined order;
- parallel group: run children together;
- stagger: derive child start offsets deterministically;
- timeline: shared time source and child coordination;
- event queue: started, repeated, completed, cancelled, failed and target-invalidated events;
- callback delivery happens on the caller-selected executor, never from a hidden render/device thread.

### Property families

Animate transform/position/scale/rotation, bounds, opacity, RGBA color, gradient-stop values, stroke/fill attributes, numeric layout values, vector components and supported compatible path geometry. Every animatable property must specify interpolation rules or explicitly say it is discrete/non-interpolable.

## 8. Rendering and CPU/GPU separation

The logical scene and sampled animation must be usable with:
- CPU-only fallback for low capability, unavailable/lost GPU device, tests and remote sessions;
- GPU-backed renderer for supported vector geometry and composition;
- renderer-selected caching and batching;
- device-loss recovery by discarding device-bound resources and rebuilding them from the logical scene.

Do not bake HWNDs, COM interfaces, D3D resources, Direct2D brushes or DirectComposition visuals into the core data model. Rendering adapters are separate. Candidate Windows 10 paths include Direct2D geometry for vector paths and Windows composition for UI composition; select exact APIs only after a small device-loss/budgeted prototype. Microsoft recommends Windows.UI.Composition over DirectComposition for Windows 10 apps and advises removing unused visuals rather than merely setting opacity to zero: [DirectComposition best practices](https://learn.microsoft.com/en-us/windows/win32/directcomp/best-practices-for-directcomposition).

GPU acceleration is a capability, not a promise. The same scene must render through CPU fallback with documented quality/performance trade-offs. A renderer must not create an unbounded number of visuals or surfaces for every animated item.

## 9. Performance rules

- No hidden threads/timers in the value/path/track core.
- No allocation in the hot sampling path for built-in value types.
- Build/parse geometry once and reuse immutable geometry where possible.
- Use dirty-property/dirty-geometry tracking; do not rebuild paths for transform-only changes.
- Cache tessellation/geometry by immutable geometry version and renderer identity.
- Batch compatible paths/items, merge updates and avoid per-item submission where a group can be updated.
- Remove hidden/unused visuals and release stale device-bound resources.
- Keep CPU/GPU budgets in CPUv/GPUv; this document does not create a second budgeting mechanism.

## 10. Conformance and stress suite

Every statement below becomes an executable test. Rust and C share the same expected input/output vectors where their APIs overlap.

| Area | “Does X do X?” assertion |
|---|---|
| Tween boundaries | At 0%, 50%, 100%, negative and over-100% progress, samples are exactly the documented value/clamped behavior |
| Time | Zero duration, zero delay, large deltas, repeated pause/resume, seek, cancel and restart have deterministic states |
| Easing | Each curve starts at 0, ends at 1, remains finite and stays within documented bounds |
| Keyframes | Empty, duplicate, descending, exact-match and extreme timestamps behave as documented |
| Interpolation | Every built-in type reaches endpoints and midpoint; NaN/infinity handling is explicit |
| Vectors | Dot/cross/length/normalization, zero vectors and matrix composition have known expected results |
| Transform | Translation/scale/rotation order and shortest-angle interpolation match reference calculations |
| RGBA | Every channel is tested independently; alpha 0/1, transparent RGB, premultiplication and unpremultiplication never create NaN |
| Group opacity | Overlapping children prove group opacity is applied at the group-compositing boundary |
| Gradients | Stop order, duplicate offsets, alpha, clamp/spread and out-of-range sample behavior are specified |
| SVG path grammar | All command families, absolute/relative forms, repeated groups, implicit moveto lines, compact flags, signs, exponents and subpaths parse correctly |
| SVG invalid input | Truncated commands, bad flags, negative radii, unsupported commands, non-finite numbers and oversized input produce stable errors |
| SVG importer | ViewBox, preserveAspectRatio, group inheritance, defs/use cycles, transforms, gradients and opacity yield known scene snapshots |
| Security | External references/scripts/foreign objects are refused; depth, byte, node and path limits cannot be bypassed |
| Morph | Compatible topology interpolates; incompatible topology returns a defined error/fallback rather than corrupt geometry |
| Lifecycle | Disposing a scene/renderer releases only owned resources and leaves caller-owned assets valid |
| Renderer parity | CPU and GPU backends produce equivalent logical geometry, transforms and alpha within a documented tolerance |
| Device loss | GPU resource invalidation leads to CPU fallback or controlled rebuild without corrupting the logical scene |

Use unit tests, property-based tests, fuzzing of parsers, differential Rust/C contract tests, golden scene snapshots, GPU/CPU comparison tests and performance regression benchmarks. Parser fuzzing must include arbitrary bytes/UTF-8, not only hand-authored valid strings.

## 11. Beta release gates

Do not call the animation API beta-complete or publish a stable beta package until:
1. Public exports, signatures, ownership and errors are documented.
2. The SVG core profile and unsupported-feature report are implemented and tested.
3. Alpha, color-space and group-opacity semantics are frozen.
4. Timelines, sequences, parallel groups, conflict behavior and cancellation/event semantics are covered.
5. Rust and C conformance suites pass on Windows toolchains.
6. CPU fallback works and a GPU renderer has device-loss and budget-pressure coverage.
7. API examples compile, and docs do not claim features that have only been planned.

The current `0.2.0-beta.1` manifest value is a prerelease label only; it is not evidence that these gates have passed. The Rust vector/SVG path-data modules are exported in the library-foundation branch and have unit coverage for command normalization, compact flags, malformed input and parser size/command limits. Full SVG document import, alpha/color-space/group-opacity semantics, C/Rust scene/parser parity, property/fuzz coverage and renderer conformance remain open before consumers can rely on the broader API.
