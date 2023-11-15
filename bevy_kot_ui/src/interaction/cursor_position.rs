//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Helper system param for accessing a cursor's world position.
#[derive(SystemParam)]
pub struct CursorPos<'w, 's, C: LunexCursor>
{
    cursor: Query<'w, 's, &'static Cursor, (With<C>, Without<Disabled>)>,  //todo: Option<InFocusedWindow>
}

impl<'w, 's, C: LunexCursor> CursorPos<'w, 's, C>
{
    /// Check if a cursor is available in the focused window.
    pub fn available(&self) -> bool
    {
        self.cursor.is_empty()
    }

    /// Get the cursor's world position in the focused window.
    ///
    /// Returns `None` if the cursor doesn't exist or is disabled.
    pub fn get_world(&self) -> Option<Vec2>
    {
        let Ok(cursor) = self.cursor.get_single() else { return None; };
        Some(*cursor.position_world())
    }

    /// Get the cursor's screen position in the focused window.
    ///
    /// Returns `None` if the cursor doesn't exist or is disabled.
    pub fn get_screen(&self) -> Option<Vec2>
    {
        let Ok(cursor) = self.cursor.get_single() else { return None; };
        Some(*cursor.position_screen())
    }

    //todo: request position for specific window
}

//-------------------------------------------------------------------------------------------------------------------
