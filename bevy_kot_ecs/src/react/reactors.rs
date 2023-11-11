//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use core::any::TypeId;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn prepare_reactor<I, O, S, Marker>(commands: &mut Commands, callback_id: u64, reactor: S) -> SysId
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
    S: IntoSystem<I, O, Marker> + Send + Sync + 'static,
{
    let sys_id = SysId::new_raw::<ReactCallback<S>>(callback_id);
    commands.add(move |world: &mut World| register_named_system(world, sys_id, reactor));
    sys_id
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tag for tracking despawns of entities with despawn reactors.
#[derive(Component)]
struct DespawnTracker
{
    parent   : Entity,
    notifier : crossbeam::channel::Sender<Entity>,
}

impl Drop for DespawnTracker
{
    fn drop(&mut self)
    {
        let _ = self.notifier.send(self.parent);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_despawn_tracker(
    In((entity, notifier)) : In<(Entity, crossbeam::channel::Sender<Entity>)>,
    world                  : &mut World
){
    // try to get the entity
    // - if entity doesn't exist, then notify the reactor in case we have despawn reactors waiting
    let Some(mut entity_mut) = world.get_entity_mut(entity)
    else
    {
        let _ = notifier.send(entity);
        return;
    };

    // leave if entity already has a despawn tracker
    // - we don't want to accidentally trigger `DespawnTracker::drop()` by replacing the existing component
    if entity_mut.contains::<DespawnTracker>() { return; }

    // insert a new despawn tracker
    entity_mut.insert(DespawnTracker{ parent: entity, notifier });
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Add reactor to an entity. The reactor will be invoked when the event occurs on the entity.
fn register_entity_reactor<C: ReactComponent>(
    In((
        rtype,
        entity,
        sys_id
    ))                  : In<(EntityReactType, Entity, SysId)>,
    mut commands        : Commands,
    mut entity_reactors : Query<&mut EntityReactors>,
){
    // callback adder
    let add_callback_fn =
        move |entity_reactors: &mut EntityReactors|
        {
            let callbacks = match rtype
            {
                EntityReactType::Insertion => entity_reactors.insertion_callbacks.entry(TypeId::of::<C>()).or_default(),
                EntityReactType::Mutation  => entity_reactors.mutation_callbacks.entry(TypeId::of::<C>()).or_default(),
                EntityReactType::Removal   => entity_reactors.removal_callbacks.entry(TypeId::of::<C>()).or_default(),
            };
            callbacks.push(sys_id);
        };

    // add callback to entity
    match entity_reactors.get_mut(entity)
    {
        Ok(mut entity_reactors) => add_callback_fn(&mut entity_reactors),
        _ =>
        {
            let Some(mut entity_commands) = commands.get_entity(entity) else { return; };

            // make new reactor tracker for the entity
            let mut entity_reactors = EntityReactors::default();

            // add callback and insert to entity
            add_callback_fn(&mut entity_reactors);
            entity_commands.insert(entity_reactors);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors with [`ReactCommands`].
pub trait ReactorRegistrator
{
    /// Reactor input.
    type Input;

    /// Register a reactor with [`ReactCommands`].
    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor  : impl IntoSystem<Self::Input, (), Marker> + Send + Sync + 'static
    ) -> RevokeToken;
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactComponent`] insertions on any entity.
/// - Reactor takes the entity the component was inserted to.
pub struct Insertion<C: ReactComponent>(PhantomData<C>);
impl<C: ReactComponent> Default for Insertion<C> { fn default() -> Self { Self(PhantomData::default()) } }

impl<C: ReactComponent> ReactorRegistrator for Insertion<C>
{
    type Input = Entity;

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<Entity, (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        cache.register_insertion_reactor::<C>(sys_id)
    }
}

/// Obtain a [`Insertion`] reactor registration handle.
pub fn insertion<C: ReactComponent>() -> Insertion<C> { Insertion::default() }

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactComponent`] mutations on any entity.
/// - Reactor takes the entity the component was mutation on.
pub struct Mutation<C: ReactComponent>(PhantomData<C>);
impl<C: ReactComponent> Default for Mutation<C> { fn default() -> Self { Self(PhantomData::default()) } }

impl<C: ReactComponent> ReactorRegistrator for Mutation<C>
{
    type Input = Entity;

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<Entity, (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        cache.register_mutation_reactor::<C>(sys_id)
    }
}

/// Obtain a [`Mutation`] reactor registration handle.
pub fn mutation<C: ReactComponent>() -> Mutation<C> { Mutation::default() }

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactComponent`] removals from any entity.
/// - Reactor takes the entity the component was removed from.
pub struct Removal<C: ReactComponent>(PhantomData<C>);
impl<C: ReactComponent> Default for Removal<C> { fn default() -> Self { Self(PhantomData::default()) } }

impl<C: ReactComponent> ReactorRegistrator for Removal<C>
{
    type Input = Entity;

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<Entity, (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        cache.track_removals::<C>();

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        cache.register_removal_reactor::<C>(sys_id)
    }
}

/// Obtain a [`Removal`] reactor registration handle.
pub fn removal<C: ReactComponent>() -> Removal<C> { Removal::default() }

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactComponent`] insertions on a specific entity.
/// - Does nothing if the entity does not exist.
pub struct EntityInsertion<C: ReactComponent>(Entity, PhantomData<C>);

impl<C: ReactComponent> ReactorRegistrator for EntityInsertion<C>
{
    type Input = ();

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<(), (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let entity = self.0;
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        rcommands.commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Insertion, entity, sys_id), register_entity_reactor::<C>)
            );

        RevokeToken{ reactor_type: ReactorType::EntityInsertion(entity, TypeId::of::<C>()), sys_id }
    }
}

/// Obtain a [`EntityInsertion`] reactor registration handle.
pub fn entity_insertion<C: ReactComponent>(entity: Entity) -> EntityInsertion<C>
{
    EntityInsertion(entity, PhantomData::default())
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactComponent`] mutations on a specific entity.
/// - Does nothing if the entity does not exist.
pub struct EntityMutation<C: ReactComponent>(Entity, PhantomData<C>);

impl<C: ReactComponent> ReactorRegistrator for EntityMutation<C>
{
    type Input = ();

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<(), (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let entity = self.0;
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        rcommands.commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Mutation, entity, sys_id), register_entity_reactor::<C>)
            );

        RevokeToken{ reactor_type: ReactorType::EntityMutation(entity, TypeId::of::<C>()), sys_id }
    }
}

/// Obtain a [`EntityMutation`] reactor registration handle.
pub fn entity_mutation<C: ReactComponent>(entity: Entity) -> EntityMutation<C>
{
    EntityMutation(entity, PhantomData::default())
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactComponent`] removals from a specific entity.
/// - Does nothing if the entity does not exist.
/// - If a component is removed from the entity then despawned (or removed due to a despawn) before
///   [`react_to_removals()`] is executed, then the reactor will not be scheduled.
pub struct EntityRemoval<C: ReactComponent>(Entity, PhantomData<C>);

impl<C: ReactComponent> ReactorRegistrator for EntityRemoval<C>
{
    type Input = ();

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<(), (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let entity = self.0;
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        cache.track_removals::<C>();

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        rcommands.commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Removal, entity, sys_id), register_entity_reactor::<C>)
            );

        RevokeToken{ reactor_type: ReactorType::EntityRemoval(entity, TypeId::of::<C>()), sys_id }
    }
}

/// Obtain a [`EntityRemoval`] reactor registration handle.
pub fn entity_removal<C: ReactComponent>(entity: Entity) -> EntityRemoval<C>
{
    EntityRemoval(entity, PhantomData::default())
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for entity despawns.
/// - Returns [`None`] if the entity does not exist.
pub struct Despawn(Entity);

impl ReactorRegistrator for Despawn
{
    type Input = ();

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<(), (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let entity = self.0;
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        // if the entity doesn't exist, return a dummy revoke token
        let Some(_) = rcommands.commands.get_entity(entity)
        else { return RevokeToken{ reactor_type: ReactorType::Despawn(entity), sys_id: SysId::new_raw::<()>(0u64) }; };

        // add despawn tracker
        let notifier =  cache.despawn_sender();
        rcommands.commands.add(move |world: &mut World| syscall(world, (entity, notifier), add_despawn_tracker));

        // register despawn reactor
        cache.register_despawn_reactor(
                entity,
                CallOnce::new(
                    move |world|
                    {
                        let mut system = IntoSystem::into_system(reactor);
                        system.initialize(world);
                        system.run((), world);
                        system.apply_deferred(world);
                    }
                ),
            )
    }
}

/// Obtain a [`Despawn`] reactor registration handle.
pub fn despawn(entity: Entity) -> Despawn { Despawn(entity) }

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for [`ReactResource`] mutations.
pub struct ResourceMutation<R: ReactResource>(PhantomData<R>);
impl<R: ReactResource> Default for ResourceMutation<R> { fn default() -> Self { Self(PhantomData::default()) } }

impl<R: ReactResource> ReactorRegistrator for ResourceMutation<R>
{
    type Input = ();

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<(), (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        cache.register_resource_mutation_reactor::<R>(sys_id)
    }
}

/// Obtain a [`ResourceMutation`] reactor registration handle.
pub fn resource_mutation<R: ReactResource>() -> ResourceMutation<R> { ResourceMutation::default() }

//-------------------------------------------------------------------------------------------------------------------

/// Reactor registration handle for events.
/// - Reactions only occur for events sent via [`ReactCommands::<E>::send()`].
pub struct Event<E: Send + Sync + 'static>(PhantomData<E>);
impl<E: Send + Sync + 'static> Default for Event<E> { fn default() -> Self { Self(PhantomData::default()) } }

impl<E: Send + Sync + 'static> ReactorRegistrator for Event<E>
{
    type Input = ();

    fn register<Marker>(self,
        rcommands : &mut ReactCommands,
        reactor   : impl IntoSystem<(), (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = rcommands.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = prepare_reactor(&mut rcommands.commands, cache.next_callback_id(), reactor);
        cache.register_event_reactor::<E>(sys_id)
    }
}

/// Obtain a [`Event`] reactor registration handle.
pub fn event<E: Send + Sync + 'static>() -> Event<E> { Event::default() }

//-------------------------------------------------------------------------------------------------------------------
