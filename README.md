# Koe's Bevy toolkit

A collection of bevy tools.



## Contents

### ECS

- System callers for invoking systems as if they were functions (requires `&mut World`).
- Entity callbacks.
- Utilities for adding/removing components from entities (requires `&mut World`).
- Reactive framework managed by [`ReactCommands`](bevy_kot::prelude::ReactCommands): [`React`](bevy_kot::prelude::React) components, [`ReactRes`](bevy_kot::prelude::ReactRes) resources, and reactive events (with [`ReactEventReader`](bevy_kot::prelude::ReactEventReader)). See [the documentation](/bevy_kot_ecs/README.md) for more information.
- [`AutoDespawner`](bevy_kot::prelude::AutoDespawner) resource for garbage collecting entities.


### UI

- [`StyleStack`](bevy_kot::prelude::StyleStack) provides style inheritance, which is especially useful for setting up and overriding prefab styles. You `style_stack.add()` a style to the current 'frame', and it will be available for all child frames. Then you `style_stack.push()/.pop()` to add/remove frames. At startup you can initialize the stack with a bundle of default styles, then when building a UI branch you can 'unwrap' styles with `style_stack.add(style_stack.style::<X>().my_inner_style.clone());` (e.g. if you need a special font for some widget, you pull that font onto the stack by unwrapping your widget's special style which was inserted on init).
- `UiBuilder` bundles useful system parameters for building UI: a [`ReactCommands`](bevy_kot::prelude::ReactCommands), a lunex ui tree handle, the bevy asset server, a despawner utility for entity GC, and the [`StyleStack`](bevy_kot::prelude::StyleStack) resource. The builder exposes `div()` and `div_rel()` which are convenience methods for managing `StyleStack` frames.
- [`InteractiveElementBuilder`](bevy_kot::prelude::InteractiveElementBuilder) lets you add interaction callbacks to an entity, which are auto-invoked by an interaction pipeline system. It is quite opinionated and assumes you are using `bevy_lunex`, but has a fairly large API and should work as-is for most normal use-cases (it needs a major refactor to unlock the remaining use-cases). Unlike `bevy_mod_picking` where you add callbacks ad hoc, `InteractiveElementBuilder` adds callbacks 'all at once', enabling more built-in functionality. The builder's supporting code makes it possible to define different interaction sources (currently all hit tests are tied to lunex widgets, pending a refactor \[this is the biggest usability issue at the moment\]).


### Misc

- [`FpsTracker`](bevy_kot::prelude::FpsTracker) resource with plugin [`FpsTrackerPlugin`](bevy_kot::prelude::FpsTrackerPlugin).


### Utils

- [`Sender`](bevy_kot::prelude::Sender)/[`Receiver`](bevy_kot::prelude::Receiver) and [`IoSender`](bevy_kot::prelude::IoSender)/[`IoReceiver`](bevy_kot::prelude::IoReceiver) unbounded MPMC channels.



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
