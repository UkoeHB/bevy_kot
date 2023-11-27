//local shortcuts
use crate::*;
use bevy_kot_derive::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(StyleBundle)]
pub struct EditorStyleBundle
{
    //style for visibility toggle button (appearance, position and size)
    //style for layout of default window
    //style for editor windows
    //style for editor basic buttons
    //style for editor text
}

//-------------------------------------------------------------------------------------------------------------------

/// Prepares the editor framework.
pub struct EditorPlugin<Ui: LunexUi>
{
    pub styles: EditorStyleBundle,
}

impl<Ui: LunexUi> EditorPlugin<Ui>
{
    pub fn new(styles: EditorStyleBundle) -> Self
    {
        Self{ styles }
    }
}

impl<Ui: LunexUi> EditorPlugin<Ui> for Plugin
{
    fn build(app: &mut App) -> Self
    {
        //setup editor UI tree
        //editor visibility toggle button
        //default editor window
    }
}

//-------------------------------------------------------------------------------------------------------------------
