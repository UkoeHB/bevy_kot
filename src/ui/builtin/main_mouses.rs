//local shortcuts
use crate::ui::builtin::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Use to tag the main mouse cursor.
#[derive(Component, Default, Copy, Clone, Debug)]
pub struct MainMouseCursor;
impl PlainMouseCursor for MainMouseCursor {}

//-------------------------------------------------------------------------------------------------------------------

/// Mouse left button interaction source.
/// - Uses the main mouse cursor to target the main UI in a given OS window.
pub type MouseLButtonMain = MouseLButton<MainUI, MainMouseCursor>;

/// Mouse right button interaction source.
/// - Uses the main mouse cursor to target the main UI in a given OS window.
pub type MouseRButtonMain = MouseRButton<MainUI, MainMouseCursor>;

//-------------------------------------------------------------------------------------------------------------------
