# Changelog

## [0.3.0]

### Added

- `ReactRes` and `ReactResMut` system params that mimic `Res`/`ResMut`, with corresponding `World`/`App`/`Commands` methods added via extension traits.
- `ReactResource` custom derive that mimics `Resource`.
- `ReactComponent` custom derive that mimics `Component`.
- `ReactCommands` reactor registration now accepts `IntoSystem` callbacks.
- Added `named_syscall_direct()` and `register_named_syscall()`.

### Changed

- `ReactCommands::on_resource_mutation()` now takes a type that implements `ReactResource`.
- `EventRevokeToken` was removed in favor of unified `RevokeToken`s for all reactors.
- Named systems are now mapped to both the input id and the system type, instead of just the input id. This allows the internal named systems cache to not be parameterized by the system type, which makes it easier to access.


## [0.2.0]

### Changed

- `StyleStack::get()` now returns an Arc instead of reference.


## [0.1.0]

### Changed

- Try to fix dependency issues with downstream projects. Start using minor version number.


## [0.0.12]

### Changed

- Refactored into a workspace.
- Added `builtin_ui` default feature.


## [0.0.11]

### Changed

- API cleanup. `UiBuilderCtx` -> `UiBuilder`, etc.


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
