# Changelog

## [0.11.0]

### Changed

- Rename: `ReactEvents` -> `ReactEventReader`.


## [0.10.3]

### Fixed

- `ReactCommands::once()` no longer returns `Result`.


## [0.10.2]

### Added

- Added `ReactCommands::once()` for reactors that should only run once.


## [0.10.1]

### Fixed

- Improved `StyleStack` panic messages.
- Interaction pipelines now force-update the lunex cursor position before running.


## [0.10.0]

### Added

- Added `bevy_kot_misc` crate.

### Changed

- Moved `FpsTracker` to `bevy_kot_misc`.
- Changed `FpsTracker` to a reactive resource.


## [0.9.2]

### Changed

- Removed unused dependency.


## [0.9.0]

### Changed

- Update to Bevy v0.12, `bevy_lunex` v0.0.9.
- Rename: `IoReceiver::try_next()` -> `IoReceiver::try_recv()`.
- Rename: `UIInteractionBarrier` -> `UiInteractionBarrier`.
- Rename: `FPSTracker` -> `FpsTracker`.


## [0.8.0]

### Added

- `UiBuilder::div_rel()` makes it simpler to write divs for relative widgets.
- `AutoDespawner` resource for garbage collecting entities.
- `Sender`/`Receiver` unbounded MPMC channel and `IoSender`/`IoReceiver` unbounded MPMC channel.
- `InteractiveElementBuilder::build()` now requires an `AutoDespawner` reference.
- `InteractiveElementBuilder::spawn_with()` takes a `UiBuilder`.

### Changed

- Rename: `MainUI` -> `MainUi`.
- Rename: `LunexUI` -> `LunexUi`.
- Improved callback cleanup handling in react framework and interactive element builder.


## [0.7.0]

### Added

- `StyleStack::edit()` and `UiBuilder::edit_style()` can be used to edit an existing style and add the edited version to the current style frame.

### Changed

- `ReactCommands::on()` can now register a reactor for multiple reaction triggers.
- `ReactCommands` despawn reactors must now be registered with `ReactCommands::on_despawn()`.
- `UiBuilder::get()/add()` -> `UiBuilder::get_style()/add_style()`


## [0.6.0]

### Changed

- The first time `ReactEvents::next()` is invoked by a reactor system, it always returns the first event sent after the system was registered. All older events will be ignored.
- Improved `ReactEvents` API.


## [0.5.0]

### Added

- `ReactEvents` acts like `EventReader`. Reactive events can be registered on `App`s with `add_react_event()`.

### Changed

- You no longer need to implement `Event` on reactive event data.


## [0.4.0]

### Changed

- Refactored reactive events to use Bevy `Event` for better ergonomics when writing event reactors.


## [0.3.1]

### Changed

- Fixed dependencies.


## [0.3.0]

### Added

- `ReactRes` and `ReactResMut` system params that mimic `Res`/`ResMut`, with corresponding `World`/`App`/`Commands` methods added via extension traits.
- `ReactResource` custom derive that mimics `Resource`.
- `ReactComponent` custom derive that mimics `Component`.
- `ReactCommands` reactor registration now accepts `IntoSystem` callbacks.
- `InteractiveElementBuilder` callbacks now implement `IntoSystem`.
- Added `named_syscall_direct()`, `register_named_syscall()`, and `register_named_syscall_from()`.

### Changed

- `ReactCommands` now requires reactive types implement `ReactComponent` and `ReactResource`.
- `EventRevokeToken` was removed in favor of unified `RevokeToken`s for all reactors.
- Named systems are now mapped to both the input id and the system type, instead of just the input id. This allows the internal named systems cache to not be parameterized by the system type, which makes it easier to access.
- `InteractiveElementBuilder` callbacks no longer take the cursor world position as input. Use `CursorPos` system parameter to access a cursor's position instead.


## [0.2.0]

### Changed

- `StyleStack::get()` now returns an Arc instead of reference.


## [0.1.0]

### Changed

- Try to fix dependency issues with downstream projects.
- Start using minor version number.


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
