//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Used to tag the main UI for a given OS window.
#[derive(Component, Default, Copy, Clone, Debug)]
pub struct MainUi;
impl LunexUi for MainUi {}

//-------------------------------------------------------------------------------------------------------------------
