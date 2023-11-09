//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_fn_plugin::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Prepares react framework so that reactors may be registered with [`ReactCommands`].
/// - Does NOT schedule any component removal or entity despawn reactor systems. You must schedule those yourself!
/// 
/// WARNING: If reactivity is implemented natively in Bevy, then this implementation will become obsolete.
#[bevy_plugin]
pub fn ReactPlugin(app: &mut App)
{
    app.init_resource::<ReactCache>();
}

//-------------------------------------------------------------------------------------------------------------------
