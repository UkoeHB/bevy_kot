# Koe's Bevy toolkit

A collection of bevy tools.



## Contents

### UI

- Interactive element builder for [`bevy_lunex`], with a robust backend for handling arbitrary interaction sources.
- `StyleStack` utility to enable style cascading/overriding when building a UI tree.
- `UiBuilder` utility for building UI trees. It automatically manages `StyleStack` frames.


### ECS

- System callers for invoking systems as if they were functions (requires `&mut World`).
- Entity callbacks.
- Utilities for adding/removing components from entities (requires `&mut World`).
- Reactive framework managed by [`ReactCommands`]: [`React`] components, [`ReactRes`] resources, and reactive events (with [`ReactEvents`] system param that mimics `EventReader`).
- [`AutoDespawner`] resource for garbage collecting entities.


### Miscellaneous

- [`FPSTracker`] resource with plugin [`FPSTrackerPlugin`].
- [`Sender`]/[`Receiver`] unbounded MPMC channel and [`IoSender`]/[`IoReceiver`] unbounded MPMC channel.




## Bevy compatability

| bevy | `bevy_kot`     |
|------|----------------|
| 0.11 | 0.0.1 - master |




## `bevy_lunex` compatability

| lunex | `bevy_kot`     |
|-------|----------------|
| 0.0.6 | 0.0.2 - master |
| 0.0.5 | 0.0.1          |
