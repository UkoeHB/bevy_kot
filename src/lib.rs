//documentation
#![doc = include_str!("../README.md")]

//API exports
pub mod prelude
{
    pub use bevy_kot_ecs::*;
    pub use bevy_kot_ui::*;
    pub use bevy_kot_utils::*;

    #[cfg(feature = "builtin_ui")]
    pub use bevy_kot_ui::builtin::*;  //todo: default feature?
}
