//local shortcuts
use crate::{*, Style};

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Dummy style used to hide styles in the stack.
struct DummyStyle;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// A style stack resource parameterized by the associated UI tree's type.
#[derive(Resource, Default)]
pub struct StyleStackRes<Ui: LunexUI>(StyleStack, PhantomData<Ui>);

impl<Ui: LunexUI> Deref for StyleStackRes<Ui> { type Target = StyleStack; fn deref(&self) -> &Self::Target { &self.0 }}
impl<Ui: LunexUI> DerefMut for StyleStackRes<Ui> { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }}

//-------------------------------------------------------------------------------------------------------------------

/// Manages a stack of styles.
///
/// The 'root stack frame' is implicit and cannot be popped. Styles in the root frame are permanent unless you call
/// [`StyleStack::clear()`].
#[derive(Default, Clone)]
pub struct StyleStack
{
    /// Registry of per-style stacks.
    styles: HashMap<TypeId, Vec<Arc<dyn Any + Send + Sync + 'static>>>,

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
            let Some(style_stack) = self.styles.get_mut(&style_id) else { continue; };
            style_stack.pop();
        }

        // cache the buffer
        last_frame.clear();
        self.buffers.push(last_frame);
    }

    /// Clear all contents of the style stack.
    ///
    /// Equivalent to `*stack = StyleStack::default()`.
    pub fn clear(&mut self)
    {
        *self = StyleStack::default();
    }

    /// Add a style bundle to the top of the current stack frame.
    ///
    /// The styles in this bundle will remain active until the current stack frame is popped. Bundles
    /// added after this bundle in the current stack frame will override the styles in this bundle.
    ///
    /// Note that if multiple instances of the same style are found in a bundle, only the **last**
    /// instance will be accessible by [`StyleStack::get()`].
    pub fn add(&mut self, bundle: impl StyleBundle)
    {
        let mut func =
            |style: Arc<dyn Any + Send + Sync + 'static>|
            {
                self.insert((&*style).type_id(), style);
            };
        bundle.get_styles(&mut func);
    }

    /// Hide a specific style for the current stack frame.
    pub fn hide<S: Style>(&mut self)
    {
        let type_id = TypeId::of::<S>();
        self.insert(type_id, Arc::new(DummyStyle));
    }

    /// Get a specific style.
    ///
    /// The returned style will be taken from the top of the style stack.
    ///
    /// Returns `None` if there is no style entry or if the style was hidden with [`StyleStack::hide()`].
    pub fn get<S: Style>(&self) -> Option<&S>
    {
        self
            .styles
            .get(&TypeId::of::<S>())
            .map_or(
                None,
                |stack|
                stack.last().map_or(None, |style| style.downcast_ref())
            )
    }

    fn insert(&mut self, type_id: TypeId, style: Arc<dyn Any + Send + Sync + 'static>)
    {
        self.styles.entry(type_id).or_default().push(style);
        if let Some(top) = self.stack.last_mut()
        {
            top.push(type_id);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
