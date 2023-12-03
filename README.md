# Koe's Bevy toolkit

A collection of bevy tools.



## Contents

### ECS

- System callers for invoking systems as if they were functions (requires `&mut World`).
- Entity callbacks.
- Utilities for adding/removing components from entities (requires `&mut World`).
- Reactive framework managed by [`ReactCommands`]: [`React`] components, [`ReactRes`] resources, and reactive events (with [`ReactEvents`] system param that mimics `EventReader`).
- [`AutoDespawner`] resource for garbage collecting entities.


### UI

- Interactive element builder for [`bevy_lunex`], with a robust backend for handling arbitrary interaction sources.
- `StyleStack` utility to enable style cascading/overriding when building a UI tree.
- `UiBuilder` utility for building UI trees. It automatically manages `StyleStack` frames.


### Utils

- [`FpsTracker`] resource with plugin [`FpsTrackerPlugin`].
- [`Sender`]/[`Receiver`] unbounded MPMC channel and [`IoSender`]/[`IoReceiver`] unbounded MPMC channel.



## Bevy compatability

| bevy | `bevy_kot`     |
|------|----------------|
| 0.12 | 0.9.0 - master |
| 0.11 | 0.0.1 - 0.8.0  |




## `bevy_lunex` compatability

| lunex | `bevy_kot`     |
|-------|----------------|
| 0.0.9 | 0.9.0 - master |
| 0.0.6 | 0.0.2 - 0.8.0  |
| 0.0.5 | 0.0.1          |
