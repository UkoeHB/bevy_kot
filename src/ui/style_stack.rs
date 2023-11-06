//local shortcuts
use super::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------

/// Identifies style structs.
//todo: require `Eq` and avoid having duplicate style structs in the cache? maybe too restrictive
pub trait Style {}

//-------------------------------------------------------------------------------------------------------------------

/// Collection of styles.
///
/// All members of a style bundle struct must implement [`Style`].
pub trait StyleBundle
{

}

//-------------------------------------------------------------------------------------------------------------------

/// Manages a stack of styles.
pub struct StyleStack
{
    /// Registry of per-style stacks.
    styles: HashMap<TypeId, Vec<Arc<dyn Any>>>,

    /// Stack frames.
    stack: Vec<Vec<TypeId>>,

    /// Cached buffers.
    buffers: Vec<Vec<TypeId>>,
}

impl StyleStack
{
    /// Add an empty stack frame.
    pub fn push(&mut self)
    {
        self.stack.push(self.buffers.pop().unwrap_or_default());
    }

    /// Remove the latest stack frame.
    pub fn pop(&mut self)
    {
        // pop the frame
        let Some(mut last_frame) = self.stack.pop() else { return; };

        // clean up per-style stacks
        for style_id in last_frame.iter()
        {
            let style_stack = self.styles.get_mut(&style_id) else { continue; };
            style_stack.pop();
        }

        // cache the buffer
        last_frame.clear();
        self.buffers.push(last_frame);
    }

    /// Add a style bundle to the top of the current stack frame.
    ///
    /// The styles in this bundle will remain active until the current stack frame is popped.
    ///
    /// Note that if multiple instances of the same style are found in the bundle, only the **last**
    /// instance will be accessible by [`StyleStack::get()`].
    pub fn add(&mut self, bundle: impl StyleBundle)
    {
        //unpack the styles in the bundle and push them into the cache
    }

    /// Get a specific style.
    ///
    /// The returned style will be taken from the top of the style stack.
    pub fn get<S>(&self) -> Option<&S>
    {
        self
            .styles
            .get(&TypeId::of::<S>())
            .map(
                |stack|
                stack
                    .last()
                    .map(|style| style.downcast_ref().unwrap())
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------
