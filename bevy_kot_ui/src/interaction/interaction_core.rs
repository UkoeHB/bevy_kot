//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::{Command, SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts
use std::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Clone, Debug)]
struct InteractionSourceRunner<S: InteractionSource> { _p: PhantomData<S> }

impl<S: InteractionSource> Default for InteractionSourceRunner<S>
{ fn default() -> Self { Self{ _p: PhantomData::default() } } }

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Components with this trait are used to tag `bevy_lunex::UiTree`s for accurate filtering in interaction pipelines.
/// - Currently only one UI per OS window may have a given tag.
pub trait LunexUI: Component {}

//-------------------------------------------------------------------------------------------------------------------

/// A `LunexCursor` represents an interface between a hardware cursor (represented by a `bevy_lunex::Cursor`),
/// and elements connected to a `bevy_lunex::UiTree` in a bevy world. You can add and remove `LunexCursor` components on
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
    fn cursor_intersects_barrier(
        cursor_world_position : Vec2,
        cursor_lunex_position : Vec2,
        ui                    : &UiTree,
        widget                : &Widget,
        widget_entity         : Entity,
        depth_limit           : Option<f32>,
        widget_depth          : f32,
        barrier_param         : &SystemParamItem<Self::BarrierParam>,
    ) -> Result<Option<f32>, ()>;

    /// Test if a cursor intersects with an element.
    fn cursor_intersects_element(
        cursor_world_position : Vec2,
        cursor_lunex_position : Vec2,
        ui                    : &UiTree,
        widget                : &Widget,
        widget_entity         : Entity,
        depth_limit           : Option<f32>,
        widget_depth          : f32,
        element_param         : &SystemParamItem<Self::ElementParam>,
    ) -> Result<Option<f32>, ()>;

    /// Test if a cursor intersects with a press home zone.
    fn cursor_intersects_press_home_zone(
        cursor_world_position : Vec2,
        cursor_lunex_position : Vec2,
        ui                    : &UiTree,
        widget                : &Widget,
        widget_entity         : Entity,
        depth_limit           : Option<f32>,
        widget_depth          : f32,
        home_zone_param       : &SystemParamItem<Self::HomeZoneParam>,
    ) -> Result<Option<f32>, ()>;
}

//-------------------------------------------------------------------------------------------------------------------

/// An interaction source represents a source of interactions (hovers and clicks) for `bevy_lunex::UiTrees` with a
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
    type LunexUI: LunexUI;
    type LunexCursor: LunexCursor;

    fn just_clicked(&self, source: &SystemParamItem<Self::SourceParam>) -> bool;
    fn is_clicked(&self, source: &SystemParamItem<Self::SourceParam>) -> bool;
    fn just_unclicked(&self, source: &SystemParamItem<Self::SourceParam>) -> bool;
}

//-------------------------------------------------------------------------------------------------------------------

/// Globally enable an interaction source (only works if the source was registered).
/// - Use `DisableInteractionSource` to disable.
pub struct EnableInteractionSource<S: InteractionSource> { _p: PhantomData<S> }

impl<S: InteractionSource> Command for EnableInteractionSource<S>
{ fn apply(self, world: &mut World) { world.init_resource::<InteractionSourceRunner<S>>(); } }

//-------------------------------------------------------------------------------------------------------------------

/// Globally disable an interaction source.
/// - Use `EnableInteractionSource` to re-enable.
pub struct DisableInteractionSource<S: InteractionSource> { _p: PhantomData<S> }

impl<S: InteractionSource> Command for DisableInteractionSource<S>
{ fn apply(self, world: &mut World) { world.remove_resource::<InteractionSourceRunner<S>>(); } }

//-------------------------------------------------------------------------------------------------------------------

/// Interaction barrier that applies to any cursor interacting with a Lunex UI tree.
#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct UIInteractionBarrier<U: LunexUI> { _p: PhantomData<U> }

//-------------------------------------------------------------------------------------------------------------------

/// Interaction barrier that applies to a specific cursor interacting with a Lunex UI tree.
#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct InteractionBarrier<U: LunexUI, C: LunexCursor> { _p: PhantomData<(U, C)> }

//-------------------------------------------------------------------------------------------------------------------

/// Indicates that this element may interact with the given source. Elements may have multiple interaction sources.
/// - Disabling this will prevent an element from being interacted with by the source (i.e. the interaction pipeline
///   for this source will no longer detect this element).
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct ElementInteractionSource<S: InteractionSource> { _p: PhantomData<S> }

impl<S: InteractionSource> Default for ElementInteractionSource<S>
{ fn default() -> Self { Self{ _p: PhantomData::default() } } }

//-------------------------------------------------------------------------------------------------------------------

/// Indicates that the targeter UI/cursor pair will take this element into account when determining the topmost
/// element in a UI tree intersected by the cursor.
/// - Disabling this will remove the element from consideration for intersection determinations, and will disable
///   interactions for any sources with the same UI/cursor pairing.
/// - Note that elements may be targeted by multiple UI/cursor pairs.
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct ElementInteractionTargeter<U: LunexUI, C: LunexCursor> { _p: PhantomData<(U, C)> }

impl<U: LunexUI, C: LunexCursor> Default for ElementInteractionTargeter<U, C>
{ fn default() -> Self { Self{ _p: PhantomData::default() } } }

//-------------------------------------------------------------------------------------------------------------------

/// Element that may be interacted with by a specific interaction source.
/// - It is not recommended to remove this bundle from entities. Instead use the `DisableElementInteractionSource`
///   and `DisableInteractiveElementTargeting` commands.
#[derive(Bundle, Copy, Clone, Eq, PartialEq, Debug)]
pub struct InteractiveElement<S: InteractionSource>
{
    _s : ElementInteractionSource<S>,
    _t : ElementInteractionTargeter<S::LunexUI, S::LunexCursor>,
}

impl<S: InteractionSource> Default for InteractiveElement<S>
{ fn default() -> Self { Self{ _s: ElementInteractionSource::default(), _t: ElementInteractionTargeter::default() } } }

//-------------------------------------------------------------------------------------------------------------------

/// Disable an interaction source on an entity. Does not disable targeting (use `DisableInteractiveElementTargeting`).
/// - To re-enable, add an `InteractiveElement` bundle to the element for this source (doing so will automatically
///   re-enable targeting for that source.
pub struct DisableElementInteractionSource<S: InteractionSource>
{
    entity : Entity,
    _p     : PhantomData<S>,
}

impl<S: InteractionSource> DisableElementInteractionSource<S>
{
    pub fn new(entity: Entity) -> Self { Self{ entity, _p: PhantomData::default() } }
} 

impl<S: InteractionSource> Command for DisableElementInteractionSource<S>
{
    fn apply(self, world: &mut World)
    {
        let Some(mut entity_ref) = world.get_entity_mut(self.entity)
        else
        {
            tracing::warn!("tried to disable interactive element source for a non-existent entity: {:?}", self.entity);
            return;
        };
        entity_ref.remove::<ElementInteractionSource<S>>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Enable interaction targeting on an entity.
/// - To disable, use the `DisableInteractiveElementTargeting` command.
pub struct EnableInteractiveElementTargeting<U: LunexUI, C: LunexCursor>
{
    entity : Entity,
    _p     : PhantomData<(U, C)>,
}

impl<U: LunexUI, C: LunexCursor> EnableInteractiveElementTargeting<U, C>
{
    pub fn new(entity: Entity) -> Self { Self{ entity, _p: PhantomData::default() } }
} 

impl<U: LunexUI, C: LunexCursor> Command for EnableInteractiveElementTargeting<U, C>
{
    fn apply(self, world: &mut World)
    {
        let Some(mut entity_ref) = world.get_entity_mut(self.entity)
        else
        {
            tracing::warn!("tried to enable interactive element targeting for a non-existent entity: {:?}", self.entity);
            return;
        };
        entity_ref.insert(ElementInteractionTargeter::<U, C>::default());
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Disable interaction targeting on an entity.
/// - To re-enable, use the `EnableInteractiveElementTargeting` command.
pub struct DisableInteractiveElementTargeting<U: LunexUI, C: LunexCursor>
{
    entity : Entity,
    _p     : PhantomData<(U, C)>
}

impl<U: LunexUI, C: LunexCursor> DisableInteractiveElementTargeting<U, C>
{
    pub fn new(entity: Entity) -> Self { Self{ entity, _p: PhantomData::default() } }
} 

impl<U: LunexUI, C: LunexCursor> Command for DisableInteractiveElementTargeting<U, C>
{
    fn apply(self, world: &mut World)
    {
        let Some(mut entity_ref) = world.get_entity_mut(self.entity)
        else
        {
            tracing::warn!("tried to disable interactive element targeting for a non-existent entity: {:?}", self.entity);
            return;
        };
        entity_ref.remove::<ElementInteractionTargeter<U, C>>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component tag for the press home zone (widget that overlays an element and controls press interactions).
#[derive(Component, Default, Clone, PartialEq, Debug)]
pub struct PressHomeZone(pub Widget);

//-------------------------------------------------------------------------------------------------------------------

/* Interactive element states */

#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Pressed
{
    #[default]
    Home,
    Away,
}

#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Selected;

#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Hovered;

//-------------------------------------------------------------------------------------------------------------------

/* Interactive element actions */

/// Callback added to interactive element: `CallbackWith<StartPress, Vec2>`
/// - Inputs: world position of cursor.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct StartPress;

/// Callback added to interactive element: `CallbackWith<UnPress, Vec2>`
/// - Inputs: world position of cursor.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct UnPress;

/// Callback added to interactive element: `CallbackWith<AbortPress, Vec2>`
/// - Inputs: world position of cursor.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct AbortPress;

/// Callback added to interactive element: `Callback<Select>`
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Select;

/// Callback added to interactive element: `Callback<Deselect>`
/// - Only invoked when the element is `Selected` and `with_select_toggling` is set. Intended for manual use.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Deselect;

//-------------------------------------------------------------------------------------------------------------------

/* Interactive element responders */

/// Callback invoked when an interaction source is just clicked over the entity.
///
/// Callback added to interactive element: `CallbackWith<OnClick, Vec2>`.
/// - Inputs: world position of cursor.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClick;

/// Callback invoked when an interaction source is clicking over the entity.
///
/// Callback added to interactive element: `CallbackWith<OnClickHold, Vec2>`.
/// - Inputs: world position of cursor.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClickHold;

/// Callback invoked when an interaction source is clicking over the entity's press home zone
/// when the entity is pressed.
///
/// Callback added to interactive element: `CallbackWith<OnClickHoldHome, Vec2>`.
/// - Inputs: world position of cursor.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClickHoldHome;

/// Callback invoked when an interaction source is clicking away from the entity's press home zone
/// when the entity is pressed.
///
/// Callback added to interactive element: `CallbackWith<OnClickHoldAway, (Vec2, bool)>`.
/// - Inputs: world position of cursor, true/false if element is present.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClickHoldAway;

/// Callback invoked when an interaction source just unclicked and the entity is pressed.
///
/// Callback added to interactive element: `CallbackWith<OnUnClick, (Vec2, bool)>`.
/// - Inputs: world position of cursor, true/false if element is under cursor.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnUnClick;

/// Callback invoked when an interaction source is hovering over the entity.
///
/// Callback added to interactive element: `CallbackWith<OnHover, Vec2>`.
/// - Inputs: world position of cursor.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnHover;

/// Callback invoked if an entity is [`Hovered`] but now the interaction source is away from the entity.
///
/// Callback added to interactive element: `Callback<OnUnHover>`.
/// - Only invoked when the element is `Hovered`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnUnHover;

//-------------------------------------------------------------------------------------------------------------------

/// Indicates an entity is disabled.
#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Disabled;

//-------------------------------------------------------------------------------------------------------------------

/// System set that contains all interaction pipelines.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct InteractionSourceSet;

//-------------------------------------------------------------------------------------------------------------------

pub trait RegisterInteractionSourceExt
{ fn register_interaction_source<S: InteractionSource>(&mut self, interaction_source: S) -> &mut Self; }

impl RegisterInteractionSourceExt for App
{
    /// Register an `InteractionSource` so interactive elements associated with the source can respond to interactions.
    /// - The source can be enabled/disabled with commands `EnableInteractionSource<[source]>` and
    ///   `DisableInteractionSource<[source]>`.
    /// - The source will be added as a resource to the app, allowing it to be modified dynamically (for example to update
    ///   cursor-based key bindings). If the source is removed then the associated interaction pipeline will be disabled.
    /// - If ordering matters between interaction sources, apply ordering constraints to the pertinent
    ///   `interaction_pipeline<[source]>` systems in schedule `First`.
    fn register_interaction_source<S: InteractionSource>(&mut self, interaction_source: S) -> &mut Self
    {
        self.init_resource::<InteractiveCallbackTracker>()  //todo: redundant with multiple sources
            .init_resource::<InteractionSourceRunner<S>>()
            .insert_resource(interaction_source)
            .add_systems(First,
                interaction_pipeline::<S>
                    .run_if(resource_exists::<InteractionSourceRunner<S>>())
                    .run_if(resource_exists::<S>())
                    .in_set(InteractionSourceSet)
            )
            .add_systems(Last, cleanup_interactive_callbacks)  //todo: redundant with multiple sources
    }
}

//-------------------------------------------------------------------------------------------------------------------
