//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

//standard shortcuts
use core::any::TypeId;
use std::marker::PhantomData;
use std::vec::Vec;

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
    entity_mut.insert(
            DespawnTracker{ parent: entity, notifier }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Cache of event-specific reactors.
#[derive(Resource)]
struct EventReactors<E: Send + Sync + 'static>
{
    reactors: Vec<(u64, CallbackWith<(), ReactEvent<E>>)>,
}

impl<E: Send + Sync + 'static> Default for EventReactors<E> { fn default() -> Self { Self{ reactors: Vec::new() } } }

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Add reactor to an entity. The reactor will be invoked when the event occurs on the entity.
fn register_entity_reactor<C: Send + Sync + 'static>(
    In((
        rtype,
        entity,
        callback,
        callback_id
    ))                  : In<(EntityReactType, Entity, Callback<()>, u64)>,
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
            callbacks.push((callback_id, callback));
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

fn register_event_reactor<E: Send + Sync + 'static>(
    commands    : &mut Commands,
    callback    : CallbackWith<(), ReactEvent<E>>,
    callback_id : u64,
) -> EventRevokeToken<E>
{
    commands.add(
            move |world: &mut World|
            {
                let mut event_reactors = world.get_resource_or_insert_with(|| EventReactors::<E>::default());
                event_reactors.reactors.push((callback_id, callback));
            }
        );

    EventRevokeToken::<E>{ callback_id, _p: PhantomData::default() }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to a data event.
fn react_to_data_event<E: Send + Sync + 'static>(
    In(event)      : In<E>,
    mut commands   : Commands,
    event_reactors : Option<Res<EventReactors<E>>>)
{
    // get reactors
    let Some(ref event_reactors) = event_reactors else { return; };
    if event_reactors.reactors.len() == 0 { return; }

    // queue reactions
    let event = ReactEvent::new(event);

    for (_, callback) in event_reactors.reactors.iter()
    {
        enque_reaction(&mut commands, callback.call_with(event.clone()));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Revoke an entity reactor.
fn revoke_entity_reactor(
    In((
        entity,
        rtype,
        comp_id,
        callback_id
    ))                  : In<(Entity, EntityReactType, TypeId, u64)>,
    mut commands        : Commands,
    mut entity_reactors : Query<&mut EntityReactors>
){
    // get this entity's entity reactors
    let Ok(mut entity_reactors) = entity_reactors.get_mut(entity) else { return; };

    // get cached callbacks
    let callbacks_map = match rtype
    {
        EntityReactType::Insertion => &mut entity_reactors.insertion_callbacks,
        EntityReactType::Mutation  => &mut entity_reactors.mutation_callbacks,
        EntityReactType::Removal   => &mut entity_reactors.removal_callbacks,
    };
    let Some(callbacks) = callbacks_map.get_mut(&comp_id) else { return; };

    // revoke reactor
    for (idx, (id, _)) in callbacks.iter().enumerate()
    {
        if *id != callback_id { continue; }
        let _ = callbacks.remove(idx);  //todo: consider swap_remove()
        break;
    }

    // clean up if entity has no reactors
    if !(callbacks.len() == 0) { return; }
    let _ = callbacks_map.remove(&comp_id);

    if !entity_reactors.is_empty() { return; }
    commands.get_entity(entity).unwrap().remove::<EntityReactors>();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Revoke an event reactor.
fn revoke_event_reactor<E: Send + Sync + 'static>(
    In(callback_id)    : In<u64>,
    mut commands       : Commands,
    mut event_reactors : Option<ResMut<EventReactors<E>>>
){
    // get reactors
    let Some(ref mut event_reactors) = event_reactors else { return; };

    // revoke reactor
    for (idx, (id, _)) in event_reactors.reactors.iter().enumerate()
    {
        if *id != callback_id { continue; }
        let _ = event_reactors.reactors.remove(idx);  //todo: consider swap_remove()
        break;
    }

    // cleanup if reactors is empty
    if event_reactors.reactors.len() > 0 { return; }
    commands.remove_resource::<EventReactors<E>>();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Drives reactivity.
///
/// Typically used with [`ReactPlugin`]. If [`ReactPlugin`] is not present:
/// - Adding a reactor will panic.
/// - Reactions will not be triggered.
/// - Sending a [`ReactEvent`] will do nothing.
///
///
/// ## Ordering and determinism
///
/// `ReactCommands` requires exclusive access to an internal cache, which means the order of react events is fully
/// specified. Reactors of the same type will react to an event in the order they are added, and react commands will
/// be applied in the order they were invoked (see API notes, some commands are applied immediately, and some are deferred.
/// Reactions to a reactor will always be resolved immediately after the reactor ends,
/// in the order they were queued (and so on up the reaction tree). A reactor's component removals and entity despawns
/// are queued alongside child reactions, which means a removal/despawn can only be 'seen' once its place in the queue
/// has been processed. Reactors always schedule reactions to available removals/despawns after they run, so if you have
/// [despawn A, reaction X, despawn B], and both despawns have reactions, then despawn A will be the first despawn reacted
/// to at the end of reaction X (or at end of the first leaf node of a reaction branch stemming from X), before any of X's
/// despawns.
///
/// A reaction tree is single-threaded by default (it may be multi-threaded if you manually invoke a bevy schedule within
/// the tree), so trees are deterministic. However, root-level reactive systems (systems that cause reactions but are
/// not themselves reactors) are subject to the ordering constraints of their callers (e.g. a bevy app schedule), and
/// reaction trees can only be initiated by calling [`apply_deferred()`]. This means the order that root-level reactors are
/// queued, and the order of root-level removals/despawns, is unspecified by the react framework.
///
///
/// ## Notes
///
/// A reaction tree is like a multi-layered accordion of command queues that automatically expands and resolves itself. Of
/// note, the 'current' structure of that accordion tree cannot be modified. For
/// example, you cannot add a data event reactor after an instance of a data event of that type that is below you in the
/// reaction tree and expect the new reactor will respond to that data event instance. Moreover, already-queued reactions/
/// react commands cannot be removed from the tree. However, modifications to the ECS world will be reflected in the
/// behavior of future reactors, which may effect the structure of not-yet-expanded parts of the accordion.
///
/// Component removal and entity despawn reactions can only occur if you explicitly call [`react_to_removals()`],
/// [`react_to_despawns()`], or [`react_to_all_removals_and_despawns()`]. We call those automatically in reaction trees, but
/// if a root-level reactive system doesn't cause any reactions then removals/despawns won't be handled. For that reason,
/// we recommand always pessimistically checking for removals/despawns manually after a call to `apply_deferred` after
/// root-level reactive systems.
///
/// WARNING: All ordering constraints may be thrown out the window with bevy native command batching.
///
#[derive(SystemParam)]
pub struct ReactCommands<'w, 's>
{
    pub(crate) commands : Commands<'w, 's>,
    pub(crate) cache    : Option<ResMut<'w, ReactCache>>,
}

impl<'w, 's> ReactCommands<'w, 's>
{
    /// Access [`Commands`].
    pub fn commands<'a>(&'a mut self) -> &'a mut Commands<'w, 's>
    {
        &mut self.commands
    }

    /// Insert a [`React<C>`] component to the specified entity.
    /// - Reactions are enacted after `apply_deferred` is invoked.
    /// - Does nothing if the entity does not exist.
    //todo: consider more ergonomic entity access, e.g. ReactEntityCommands
    pub fn insert<C: Send + Sync + 'static>(&mut self, entity: Entity, component: C)
    {
        let Some(mut entity_commands) = self.commands.get_entity(entity) else { return; };
        entity_commands.insert( React{ entity, component } );

        let Some(ref mut cache) = self.cache else { return; };
        cache.react_to_insertion::<React<C>>(&mut self.commands, entity);
    }

    /// Send a react event.
    /// - Reactions are enacted after `apply_deferred` is invoked.
    /// - Any reactors to this event will obtain a [`ReactEvent`] wrapping the event value.
    pub fn send<E: Send + Sync + 'static>(&mut self, event: E)
    {
        if self.cache.is_none() { return; }
        self.commands.add(move |world: &mut World| { syscall(world, event, react_to_data_event); });
    }

    /// Trigger resource mutation reactions.
    ///
    /// Useful for initializing state after a reactor is registered.
    pub fn trigger_resource_mutation<R: Reactive + Resource + Send + Sync + 'static>(&mut self)
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        cache.react_to_resource_mutation::<R>(&mut self.commands);
    }

    /// Revoke a reactor.
    /// - Entity reactors: registered after `apply_deferred` is invoked.
    /// - Component, despawn, resource reactors: registered immediately.
    pub fn revoke(&mut self, token: RevokeToken)
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let cb_id = token.callback_id;
        match token.reactor_type
        {
            ReactorType::EntityInsertion(entity, comp_id) =>
            {
                self.commands.add(
                        move |world: &mut World|
                        syscall(world, (entity, EntityReactType::Insertion, comp_id, cb_id), revoke_entity_reactor)
                    );
            }
            ReactorType::EntityMutation(entity, comp_id) =>
            {
                self.commands.add(
                        move |world: &mut World|
                        syscall(world, (entity, EntityReactType::Mutation, comp_id, cb_id), revoke_entity_reactor)
                    );
            }
            ReactorType::EntityRemoval(entity, comp_id) =>
            {
                self.commands.add(
                        move |world: &mut World|
                        syscall(world, (entity, EntityReactType::Removal, comp_id, cb_id), revoke_entity_reactor)
                    );
            }
            ReactorType::ComponentInsertion(comp_id) =>
            {
                cache.revoke_component_reactor(EntityReactType::Insertion, comp_id, cb_id);
            }
            ReactorType::ComponentMutation(comp_id) =>
            {
                cache.revoke_component_reactor(EntityReactType::Mutation, comp_id, cb_id);
            }
            ReactorType::ComponentRemoval(comp_id) =>
            {
                cache.revoke_component_reactor(EntityReactType::Removal, comp_id, cb_id);
            }
            ReactorType::Despawn(entity) =>
            {
                cache.revoke_despawn_reactor(entity, cb_id);
            }
            ReactorType::ResourceMutation(res_id) =>
            {
                cache.revoke_resource_mutation_reactor(res_id, cb_id);
            }
        }
    }

    /// Revoke an event reactor.
    /// - Reactor is registered after `apply_deferred` is invoked.
    pub fn revoke_event_reactor<E: Send + Sync + 'static>(&mut self, token: EventRevokeToken<E>)
    {
        self.commands.add(move |world: &mut World| syscall(world, token.callback_id, revoke_event_reactor::<E>));
    }

    /// React when a [`React`] component is inserted on any entity.
    /// - Reactor is registered immediately.
    /// - Reactor takes the entity the component was inserted to.
    pub fn on_insertion<C: Reactive + Component + Send + Sync + 'static>(
        &mut self,
        reactor: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        cache.register_insertion_reactor::<C>(CallbackWith::new(reactor))
    }

    /// React when a [`React`] component is inserted on a specific entity.
    /// - Reactor is registered after `apply_deferred` is invoked.
    /// - Does nothing if the entity does not exist.
    pub fn on_entity_insertion<C: Reactive + Component + Send + Sync + 'static>(
        &mut self,
        entity  : Entity,
        reactor : impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        let callback_id = cache.next_callback_id();
        self.commands.add(
                move |world: &mut World|
                syscall(
                        world,
                        (EntityReactType::Insertion, entity, Callback::new(reactor), callback_id),
                        register_entity_reactor::<C>
                    )
            );

        RevokeToken{ reactor_type: ReactorType::EntityInsertion(entity, TypeId::of::<C>()), callback_id }
    }

    /// React when a [`React`] component is mutated on any entity.
    /// - Reactor is registered immediately.
    /// - Reactor takes the entity the component was mutated on.
    pub fn on_mutation<C: Reactive + Component + Send + Sync + 'static>(
        &mut self,
        reactor: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        cache.register_mutation_reactor::<C>(CallbackWith::new(reactor))
    }

    /// React when a [`React`] is mutated on a specific entity.
    /// - Reactor is registered after `apply_deferred` is invoked.
    /// - Does nothing if the entity does not exist.
    pub fn on_entity_mutation<C: Reactive + Component + Send + Sync + 'static>(
        &mut self,
        entity  : Entity,
        reactor : impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        let callback_id = cache.next_callback_id();
        self.commands.add(
                move |world: &mut World|
                syscall(
                        world,
                        (EntityReactType::Mutation, entity, Callback::new(reactor), callback_id),
                        register_entity_reactor::<C>
                    )
            );

        RevokeToken{ reactor_type: ReactorType::EntityMutation(entity, TypeId::of::<C>()), callback_id }
    }

    /// React when a component `C` is removed from any entity (`C` may be a [`React<T>`] or another component).
    /// - Reactor is registered immediately.
    /// - Reactor takes the entity the component was removed from.
    /// - If a component is removed from an entity then despawned (or removed due to a despawn) before
    ///   [`react_to_removals()`] is executed, then the reactor will not be scheduled.
    pub fn on_removal<C: Component>(
        &mut self,
        reactor : impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        cache.track_removals::<C>();

        cache.register_removal_reactor::<C>(CallbackWith::new(reactor))
    }

    /// React when a component `C` is removed from a specific entity (`C` may be a [`React<T>`] or another
    /// component).
    /// - Reactor is registered after `apply_deferred` is invoked.
    /// - Does nothing if the entity does not exist.
    /// - If a component is removed from the entity then despawned (or removed due to a despawn) before
    ///   [`react_to_removals()`] is executed, then the reactor will not be scheduled.
    pub fn on_entity_removal<C: Component>(
        &mut self,
        entity  : Entity,
        reactor : impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        cache.track_removals::<C>();
        let callback_id = cache.next_callback_id();
        self.commands.add(
                move |world: &mut World|
                syscall(
                        world,
                        (EntityReactType::Removal, entity, Callback::new(reactor), callback_id),
                        register_entity_reactor::<C>
                    )
            );

        RevokeToken{ reactor_type: ReactorType::EntityRemoval(entity, TypeId::of::<C>()), callback_id }
    }

    /// React when an entity is despawned.
    /// - Reactor is registered immediately.
    /// - Returns [`None`] if the entity does not exist.
    pub fn on_despawn(
        &mut self,
        entity    : Entity,
        reactonce : impl FnOnce(&mut World) -> () + Send + Sync + 'static
    ) -> Option<RevokeToken>
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        let Some(_) = self.commands.get_entity(entity) else { return None; };
        let notifier =  cache.despawn_sender();
        self.commands.add(move |world: &mut World| syscall(world, (entity, notifier), add_despawn_tracker));

        Some(cache.register_despawn_reactor(entity, CallOnce::new(reactonce)))
    }

    /// React when a [`ReactRes`] resource is mutated.
    /// - Reactor is registered immediately.
    pub fn on_resource_mutation<R: Reactive + Resource + Send + Sync + 'static>(
        &mut self,
        reactor : impl Fn(&mut World) -> () + Send + Sync + 'static
    ) -> RevokeToken
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        cache.register_resource_mutation_reactor::<R>(Callback::new(reactor))
    }

    /// React when a data event is sent.
    /// - Reactor is registered after `apply_deferred` is invoked.
    /// - Reactions only occur for data sent via [`ReactCommands::<E>::send()`].
    pub fn on_event<E: Send + Sync + 'static>(
        &mut self,
        reactor : impl Fn(&mut World, ReactEvent<E>) -> () + Send + Sync + 'static
    ) -> EventRevokeToken<E>
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        let callback_id = cache.next_callback_id();

        register_event_reactor::<E>(&mut self.commands, CallbackWith::new(reactor), callback_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------
