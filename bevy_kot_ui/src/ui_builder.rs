//local shortcuts
use crate::{*, Style};
use bevy_kot_ecs::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Context for building a UI tree.
///
/// Can be accessed via bevy `Query`s.
///
/// Note that the style stack will persist its state between queries. Styles can be added to the root style stack frame
/// in order to re-use them between UI construction systems, or you can manually add stack frames if you want custom
/// style management.
#[derive(SystemParam)]
pub struct UiBuilder<'w, 's, Ui: LunexUI>
{
    pub rcommands    : ReactCommands<'w, 's>,
    pub asset_server : ResMut<'w, AssetServer>,
    pub style_stack  : ResMut<'w, StyleStackRes<Ui>>,

    ui: Query<'w, 's, &'static mut UiTree, With<Ui>>,  //todo: what about trees in different windows?
}

impl<'w, 's, Ui: LunexUI> UiBuilder<'w, 's, Ui>
{
    /// Get `Commands`.
    pub fn commands<'a>(&'a mut self) -> &'a mut Commands<'w, 's>
    {
        self.rcommands.commands()
    }

    /// Get a reference to the builder's associated `UiTree`.
    pub fn tree<'a>(&'a mut self) -> &'a mut UiTree
    {
        self.ui.single_mut().into_inner()
    }

    /// Create a new UI tree content division.
    ///
    /// This method adds a new style stack frame before invoking the `div` callback, then pops a frame afterward.
    pub fn div<R>(&mut self, div: impl FnOnce(&mut UiBuilder<Ui>) -> R) -> R
    {
        self.style_stack.push();
        let result = (div)(self);
        self.style_stack.pop();

        result
    }

    /// Add a style bundle to the style stack.
    pub fn add_style(&mut self, bundle: impl StyleBundle)
    {
        self.style_stack.add(bundle);
    }

    /// Get a style from the style stack.
    pub fn get_style<S: Style>(&self) -> Option<Arc<S>>
    {
        self.style_stack.get::<S>()
    }

    /// Get a clone of a style from the style stack.
    pub fn get_style_clone<S: Style + Clone>(&self) -> Option<S>
    {
        self.style_stack.get_clone::<S>()
    }

    /// Edit a style on the style stack and place the updated copy in the current style frame.
    pub fn edit_style<S: Style + Clone>(&mut self, editor: impl FnOnce(&mut S)) -> Option<Arc<S>>
    {
        self.style_stack.edit::<S>(editor)
    }
}

//-------------------------------------------------------------------------------------------------------------------
