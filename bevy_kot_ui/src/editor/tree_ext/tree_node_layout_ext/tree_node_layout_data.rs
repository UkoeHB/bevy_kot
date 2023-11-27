//local shortcuts

//third-party shortcuts
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub struct TreeNodeLayoutData
{
    widget: Widget,
}

impl TreeNodeLayoutData
{
    pub fn new(widget: Widget) -> Self
    {
        Self{ widget }
    }

    pub fn get_widget(&self) -> &Widget
    {
        &self.widget
    }
}

//-------------------------------------------------------------------------------------------------------------------
