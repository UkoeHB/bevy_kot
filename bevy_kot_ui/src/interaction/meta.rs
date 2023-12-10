//local shortcuts
use crate::*;
use bevy_kot_ecs::*;

//third-party shortcuts
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_lunex::cursor_update;

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
pub struct EnableInteractiveElementTargeting<U: LunexUi, C: LunexCursor>
{
    entity : Entity,
    _p     : PhantomData<(U, C)>,
}

impl<U: LunexUi, C: LunexCursor> EnableInteractiveElementTargeting<U, C>
{
    pub fn new(entity: Entity) -> Self { Self{ entity, _p: PhantomData::default() } }
} 

impl<U: LunexUi, C: LunexCursor> Command for EnableInteractiveElementTargeting<U, C>
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
pub struct DisableInteractiveElementTargeting<U: LunexUi, C: LunexCursor>
{
    entity : Entity,
    _p     : PhantomData<(U, C)>
}

impl<U: LunexUi, C: LunexCursor> DisableInteractiveElementTargeting<U, C>
{
    pub fn new(entity: Entity) -> Self { Self{ entity, _p: PhantomData::default() } }
} 

impl<U: LunexUi, C: LunexCursor> Command for DisableInteractiveElementTargeting<U, C>
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
        self.setup_auto_despawn()
            .init_resource::<InteractionSourceRunner<S>>()
            .insert_resource(interaction_source)
            .add_systems(First,
                (
                    cursor_update,  //todo: this is duplicate work, but we need fresh cursor state
                    interaction_pipeline::<S>,
                )
                    .chain()
                    .run_if(resource_exists::<InteractionSourceRunner<S>>())
                    .run_if(resource_exists::<S>())
                    .in_set(InteractionSourceSet)
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------
