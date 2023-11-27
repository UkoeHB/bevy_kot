//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_lunex::prelude::*;

//standard shortcuts
use std::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

/// Element that may be interacted with by a specific interaction source.
/// - It is not recommended to remove this bundle from entities. Instead use the `DisableElementInteractionSource`
///   and `DisableInteractiveElementTargeting` commands.
#[derive(Bundle, Copy, Clone, Eq, PartialEq, Debug)]
pub struct InteractiveElement<S: InteractionSource>
{
    _s : ElementInteractionSource<S>,
    _t : ElementInteractionTargeter<S::LunexUi, S::LunexCursor>,
}

impl<S: InteractionSource> Default for InteractiveElement<S>
{ fn default() -> Self { Self{ _s: ElementInteractionSource::default(), _t: ElementInteractionTargeter::default() } } }

//-------------------------------------------------------------------------------------------------------------------

/// Interaction barrier that applies to any cursor interacting with a Lunex UI tree.
#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct UiInteractionBarrier<U: LunexUi> { _p: PhantomData<U> }

//-------------------------------------------------------------------------------------------------------------------

/// Interaction barrier that applies to a specific cursor interacting with a Lunex UI tree.
#[derive(Component, Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct InteractionBarrier<U: LunexUi, C: LunexCursor> { _p: PhantomData<(U, C)> }

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

/// Callback added to interactive element: `Callback<StartPress>`
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct StartPress;

/// Callback added to interactive element: `Callback<UnPress>`
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct UnPress;

/// Callback added to interactive element: `Callback<AbortPress>`
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
/// Callback added to interactive element: `Callback<OnClick>`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClick;

/// Callback invoked when an interaction source is clicking over the entity.
///
/// Callback added to interactive element: `Callback<OnClickHold>`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClickHold;

/// Callback invoked when an interaction source is clicking over the entity's press home zone
/// when the entity is pressed.
///
/// Callback added to interactive element: `Callback<OnClickHoldHome>`.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClickHoldHome;

/// Callback invoked when an interaction source is clicking away from the entity's press home zone
/// when the entity is pressed.
///
/// Callback added to interactive element: `CallbackWith<OnClickHoldAway, bool>`.
/// - Inputs: true/false if element is present.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnClickHoldAway;

/// Callback invoked when an interaction source just unclicked and the entity is pressed.
///
/// Callback added to interactive element: `CallbackWith<OnUnClick, bool>`.
/// - Inputs: true/false if element is under cursor.
/// - Only invoked when the element is `Pressed`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OnUnClick;

/// Callback invoked when an interaction source is hovering over the entity.
///
/// Callback added to interactive element: `Callback<OnHover>`.
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
pub(crate) struct ElementInteractionTargeter<U: LunexUi, C: LunexCursor> { _p: PhantomData<(U, C)> }

impl<U: LunexUi, C: LunexCursor> Default for ElementInteractionTargeter<U, C>
{ fn default() -> Self { Self{ _p: PhantomData::default() } } }

//-------------------------------------------------------------------------------------------------------------------
