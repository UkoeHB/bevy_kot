# Koe's Bevy toolkit

A collection of tools I built to make my life easier.



## Contents

### UI

- Interaction UI tools for `bevy_lunex`, with a robust backend for handling arbitrary interaction sources.


### ECS

- System callers for invoking systems as if they were functions (requires `&mut World`). I mainly use `syscall`.
- Entity callbacks.
- Utilities for adding/removing components from entities (requires `&mut World`).
- Reactive framework: `React` components, `ResReact` resources, and `ReactEvent` messages managed by `ReactCommands`.


### Miscellaneous

- `FPSTracker` resource with plugin `FPSTrackerPlugin`.



## Help Wanted

The more UI examples, the better. Please submit new UI examples highlighting specific designs or patterns. All levels of complexity are welcome, and I will not be a stickler for formatting or code quality.



## Bevy compatability

| bevy | `bevy_kot`     |
|------|----------------|
| 0.11 | 0.0.1 - master |




## `bevy_lunex` compatability

| lunex | `bevy_kot`     |
|-------|----------------|
| 0.0.6 | 0.0.2 - master |
| 0.0.5 | 0.0.1          |
