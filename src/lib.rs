//documentation
#![doc = include_str!("../README.md")]

//module tree
pub mod ecs;
pub mod misc;
pub mod ui;

//API exports
pub mod prelude
{
    pub use crate::ecs::*;
    pub use crate::misc::*;
    pub use crate::ui::*;
    pub use bevy_kot_derive::*;
}
