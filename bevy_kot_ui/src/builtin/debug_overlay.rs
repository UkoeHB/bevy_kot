//local shortcuts
use crate::builtin::*;

//third-party shortcuts
use bevy_fn_plugin::bevy_plugin;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Add outlines to all lunex widgets. (todo: only works if there is one UI tree)
/// - To use this you must copy the ui_debug_* assets from this repository into your project's `assets` directory.
#[bevy_plugin]
pub fn UIDebugOverlayPlugin(app: &mut App)
{
    app.add_plugins(LunexUiDebugPlugin2D::<MainUI>(std::marker::PhantomData::default()));
}

//-------------------------------------------------------------------------------------------------------------------
