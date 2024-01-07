//features
#![cfg_attr(docsrs, feature(doc_cfg))]

//module tree
mod interaction;
mod style;
mod style_stack;
mod ui_builder;
mod utils;

#[cfg(feature = "builtin")]
#[cfg_attr(docsrs, doc(cfg(feature = "builtin")))]
pub mod builtin;

//API exports
pub use crate::interaction::*;
pub use crate::style::*;
pub use crate::style_stack::*;
pub use crate::ui_builder::*;
pub use crate::utils::*;

pub use bevy_kot_derive::*;
