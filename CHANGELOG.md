# Changelog

## [0.0.10]

### Added

- `UiBuilderCtx` is a Bevy `SystemParam` that wraps resources necessary for building a UI tree.


## [0.0.9]

### Added

- `StyleStack` utility for style cascading/overriding when building a UI tree. Includes `Style` and `StyleBundle` derives for marking styles and collections of styles.


## [0.0.8]

### Changed

- `ReactCommands` reactor registration renames: `add_x_reactor()` -> `on_x()`.
- `ReactCommands` resource mutation reactor registration now requires `ReactRes<R>` instead of `R`.
- `ReactEvent` no longer implements `Deref` since it caused confusing results when calling `.clone()` on the event. Use `.get()` instead.


## [0.0.7]

### Changed

- `ReactCommands` component insertion and mutation reactor registration now requires `React<C>` instead of `C`.


## [0.0.6]

### Added

- `ReactCommands::trigger_resource_mutation()`


## [0.0.5]

### Added

- UI examples.

### Changed

- Cleanup.

### Fixed

- Multiple despawn reactors on the same entity now works properly.


## [0.0.4]

### Changed

- Cleanup.


## [0.0.3]

### Added

- `React` components, `ReactRes` resources, and `ReactEvent` messages with hooks inserted via `ReactCommands`.


## [0.0.2]

### Changed

- Docs cleanup.
- `InteractiveElementBuilder` API cleanup.

### Added

- `FPSTracker` resource with `FPSTrackerPlugin`.
- `10k_buttons` example (currently 14 FPS on my machine).

### Fixed

- Several `InteractiveElementBuilder` API bugs.


## [0.0.1]

### Added

- Initial release.
