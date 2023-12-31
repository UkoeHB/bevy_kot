//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Plain mouse cursor that can interact with lunex widgets.
/// - Anything that implements `PlainMouseCursor` also implements `LunexCursor`.
pub trait PlainMouseCursor: Component {}

impl<PlainCursor: PlainMouseCursor> LunexCursor for PlainCursor
{
    type BarrierParam  = ();
    type ElementParam  = ();
    type HomeZoneParam = ();

    /// Test if a cursor intersects with an interaction barrier.
    /// - Widget-only intersection test.
    fn cursor_intersects_barrier<Ui: LunexUi>(
        _cursor_screen_position : Vec2,
        cursor_world_position  : Vec2,
        ui                     : &UiTree<Ui>,
        widget                 : &Widget,
        _widget_entity         : Entity,
        depth_limit            : Option<f32>,
        widget_depth           : f32,
        _barrier_param         : &SystemParamItem<Self::BarrierParam>,
    ) -> Result<Option<f32>, ()>
    {
        cursor_intersects_widget(cursor_world_position.invert_y(), ui, widget, depth_limit, widget_depth)
    }

    /// Test if a cursor intersects with an element.
    /// - Widget-only intersection test.
    fn cursor_intersects_element<Ui: LunexUi>(
        _cursor_screen_position : Vec2,
        cursor_world_position  : Vec2,
        ui                     : &UiTree<Ui>,
        widget                 : &Widget,
        _widget_entity         : Entity,
        depth_limit            : Option<f32>,
        widget_depth           : f32,
        _element_param         : &SystemParamItem<Self::ElementParam>,
    ) -> Result<Option<f32>, ()>
    {
        cursor_intersects_widget(cursor_world_position.invert_y(), ui, widget, depth_limit, widget_depth)
    }

    /// Test if a cursor intersects with a press home zone.
    /// - Widget-only intersection test.
    fn cursor_intersects_press_home_zone<Ui: LunexUi>(
        _cursor_screen_position : Vec2,
        cursor_world_position  : Vec2,
        ui                     : &UiTree<Ui>,
        widget                 : &Widget,
        _widget_entity         : Entity,
        depth_limit            : Option<f32>,
        widget_depth           : f32,
        _home_zone_param       : &SystemParamItem<Self::HomeZoneParam>,
    ) -> Result<Option<f32>, ()>
    {
        cursor_intersects_widget(cursor_world_position.invert_y(), ui, widget, depth_limit, widget_depth)
    }
}

//-------------------------------------------------------------------------------------------------------------------
