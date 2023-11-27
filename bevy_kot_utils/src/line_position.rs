//local shortcuts

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Copy, Clone, Debug)]
pub struct LinePosition
{
    file: &'static str,
    line: u32,
}

impl LinePosition
{
    pub fn new(file: &'static str, line: u32) -> Self
    {
        Self{ file, line }
    }
}

//-------------------------------------------------------------------------------------------------------------------
