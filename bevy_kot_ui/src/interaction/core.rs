//local shortcuts
//third-party shortcuts
use bevy::ecs::system::{SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts

//-------------------------------------------------------------------------------------------------------------------

/// Components with this trait are used to tag `bevy_lunex::UiTree<Ui>`s for accurate filtering in interaction pipelines.
/// - Currently only one UI per OS window may have a given tag.
pub trait LunexUI: Component + Default {}

//-------------------------------------------------------------------------------------------------------------------

/// A `LunexCursor` represents an interface between a hardware cursor (represented by a `bevy_lunex::Cursor`),
/// and elements connected to a `bevy_lunex::UiTree<Ui>` in a bevy world. You can add and remove `LunexCursor` components on
/// an entity with `bevy_lunex::Cursor` as needed for different use-cases. If you add the `Disabled` component to
/// a cursor entity, then none of the associated `LunexCursor`s will be used to interact with interaction sources.
///
/// For intersection tests:
/// - It is recommended to first test if the cursor intersects with the widget (barrier, element, or press home zone)
///   that covers the test subject, then do more expensive checks like ray casting.
/// - Successful tests return `Ok(Some(depth))` with the lunex depth of the intersection. We return the
///   depth in order to properly adjust `depth_limit` for future tests (e.g. for 3d objects where an intersection occurs at
///   some depth relative to the object's 2D widget projection). Note that for accurate tests it is recommended for widgets
///   to rest at the forward-most point of 3D objects so that testing widget depth against the depth
///   limit will effectively rule out 3D elements (since the entire 3D element will be below the widget, so if the widget
///   is too low then the element must be as well).
///
/// Implementation notes:
/// - The associated system params must use types from `bevy::ecs::system::lifetimeless`.
pub trait LunexCursor: Component
{
    type BarrierParam: SystemParam;
    type ElementParam: SystemParam;
    type HomeZoneParam: SystemParam;

    /// Test if a cursor intersects with an interaction barrier.
    fn cursor_intersects_barrier<Ui: LunexUI>(
        cursor_screen_position : Vec2,
        cursor_world_position  : Vec2,
        ui                     : &UiTree<Ui>,
        widget                 : &Widget,
        widget_entity          : Entity,
        depth_limit            : Option<f32>,
        widget_depth           : f32,
        barrier_param          : &SystemParamItem<Self::BarrierParam>,
    ) -> Result<Option<f32>, ()>;

    /// Test if a cursor intersects with an element.
    fn cursor_intersects_element<Ui: LunexUI>(
        cursor_screen_position : Vec2,
        cursor_world_position  : Vec2,
        ui                     : &UiTree<Ui>,
        widget                 : &Widget,
        widget_entity          : Entity,
        depth_limit            : Option<f32>,
        widget_depth           : f32,
        element_param          : &SystemParamItem<Self::ElementParam>,
    ) -> Result<Option<f32>, ()>;

    /// Test if a cursor intersects with a press home zone.
    fn cursor_intersects_press_home_zone<Ui: LunexUI>(
        cursor_screen_position : Vec2,
        cursor_world_position  : Vec2,
        ui                     : &UiTree<Ui>,
        widget                 : &Widget,
        widget_entity          : Entity,
        depth_limit            : Option<f32>,
        widget_depth           : f32,
        home_zone_param        : &SystemParamItem<Self::HomeZoneParam>,
    ) -> Result<Option<f32>, ()>;
}

//-------------------------------------------------------------------------------------------------------------------

/// An interaction source represents a source of interactions (hovers and clicks) for `bevy_lunex::UiTree<Ui>s` with a
/// specific `LunexUI` tag.
///
/// To process interactions with a source, you must register it with `register_interaction_source`. Sources are
/// implemented as bevy Resources, allowing you to manage the internal state of the source at runtime.
///
/// Implementation notes:
/// - The associated type `SourceParam` must use types from `bevy::ecs::system::lifetimeless`.
pub trait InteractionSource: Resource
{
    type SourceParam: SystemParam;
    type LunexUi: LunexUi;
    type LunexCursor: LunexCursor;

    fn just_clicked(&self, source: &SystemParamItem<Self::SourceParam>) -> bool;
    fn is_clicked(&self, source: &SystemParamItem<Self::SourceParam>) -> bool;
    fn just_unclicked(&self, source: &SystemParamItem<Self::SourceParam>) -> bool;
}

//-------------------------------------------------------------------------------------------------------------------
