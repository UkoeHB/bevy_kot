//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::*;

//standard shortcuts
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

pub struct TreeNodeMeta
{
    /// Code line position where this tree node was declared.
    line_pos: LinePosition,

    /// Widget path to this node.
    path: String,

    /// Data for linked sub-extensions.
    extension_data: HashMap<TypeId, Arc<dyn Any + Send + Sync + 'static>>,

    /// Indicates if the node has been modified.
    changed: bool,
}

impl TreeNodeMeta
{
    pub fn new(line_pos: LinePosition, path: String) -> Self
    {
        Self{ line_pos, path, extension_data: HashMap::default(), changed: false }
    }

    pub fn set_extension_data<E: Send + Sync + 'static>(&mut self, extension_data: E)
    {
        self.extension_data.insert(TypeId::of::<E>(), Arc::new(extension_data));
    }

    pub fn remove_extension_data<E: Send + Sync + 'static>(&mut self) -> Option<Arc<E>>
    {
        self
            .extension_data
            .remove(&TypeId::of::<E>())
            .map_or(None, |extension_data| extension_data.downcast().ok())
    }

    pub fn get_extension_data<E>(&self) -> Option<Arc<E>>
    {
        self
            .extension_data
            .get(&TypeId::of::<E>())
            .map_or(None, |extension_data| extension_data.clone().downcast().ok())
    }

    pub fn num_extensions(&self) -> usize
    {
        self.extension_data.len()
    }

    pub fn mark_changed(&mut self)
    {
        self.changed = true;
    }

    pub fn is_changed(&self) -> bool
    {
        self.changed
    }
}

//-------------------------------------------------------------------------------------------------------------------
