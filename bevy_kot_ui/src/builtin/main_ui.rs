//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Used to tag the main UI for a given OS window.
#[derive(Component, Default, Copy, Clone, Debug)]
pub struct MainUI;
impl LunexUI for MainUI {}

//-------------------------------------------------------------------------------------------------------------------
