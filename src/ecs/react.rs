//local shortcuts
use crate::ecs::*;

//third-party shortcuts
use bevy::ecs::system::{Command, CommandQueue, SystemParam};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_fn_plugin::*;

//standard shortcuts
use core::any::TypeId;
use core::ops::Deref;
use std::marker::PhantomData;
use std::sync::Arc;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Cached command queues for react methods.
/// - We use a container of command queues in case of recursion.
#[derive(Resource, Default)]
struct ReactCommandQueue(Vec<CommandQueue>);

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Get a command queue from the react command queue cache.
fn pop_react_command_queue(world: &mut World) -> CommandQueue
{
    world.get_resource_or_insert_with(|| ReactCommandQueue::default())
        .0
        .pop()
        .unwrap_or_else(|| CommandQueue::default())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Put command queue back in react command queue cache.
fn push_react_command_queue(world: &mut World, queue: CommandQueue)
{
    world.get_resource_or_insert_with(|| ReactCommandQueue::default()).0.push(queue);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

enum EntityReactType
{
    Insertion,
    Mutation,
    Removal,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct EntityReactors
{
    insertion_callbacks : HashMap<TypeId, Vec<(u64, Callback<()>)>>,
    mutation_callbacks  : HashMap<TypeId, Vec<(u64, Callback<()>)>>,
    removal_callbacks   : HashMap<TypeId, Vec<(u64, Callback<()>)>>,
}

impl EntityReactors
{
    fn is_empty(&self) -> bool
    {
        self.insertion_callbacks.is_empty() &&
        self.mutation_callbacks.is_empty()  &&
        self.removal_callbacks.is_empty()  
    }
}

impl Default for EntityReactors
{
    fn default() -> Self
    {
        Self{
            insertion_callbacks : HashMap::new(),
            mutation_callbacks  : HashMap::new(),
            removal_callbacks   : HashMap::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ComponentReactors
{
    insertion_callbacks : Vec<(u64, CallbackWith<(), Entity>)>,
    mutation_callbacks  : Vec<(u64, CallbackWith<(), Entity>)>,
    removal_callbacks   : Vec<(u64, CallbackWith<(), Entity>)>,
}

impl ComponentReactors
{
    fn is_empty(&self) -> bool
    {
        self.insertion_callbacks.is_empty() &&
        self.mutation_callbacks.is_empty()  &&
        self.removal_callbacks.is_empty()  
    }
}

impl Default for ComponentReactors
{
    fn default() -> Self
    {
        Self{
            insertion_callbacks : Vec::new(),
            mutation_callbacks  : Vec::new(),
            removal_callbacks   : Vec::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct RemovalChecker
{
    component_id : TypeId,
    checker      : SysCall<(), Vec<Entity>, Vec<Entity>>
}

impl RemovalChecker
{
    fn new<C: Component>() -> Self
    {
        Self{
            component_id : TypeId::of::<C>(),
            checker      : SysCall::new(|world, buffer| syscall(world, buffer, collect_component_removals::<C>)),
        }
    }
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

/// Collect component removals.
///
/// Note: `RemovedComponents` acts like an event reader, so multiple invocations of this system within one tick will
/// not see duplicate removals.
fn collect_component_removals<C: Component>(
    In(mut buffer) : In<Vec<Entity>>,
    mut removed    : RemovedComponents<C>
) -> Vec<Entity>
{
    buffer.clear();
    removed.iter().for_each(|entity| buffer.push(entity));
    buffer
}

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

/// Queue a react callback, followed by a call to `apply_deferred`, then react to all removals and despawns.
/// - We want to apply any side effects or chained reactions before any sibling reactions/commands.
fn enque_reaction(commands: &mut Commands, cb: impl Command)
{
    commands.add(
            move |world: &mut World|
            {
                cb.apply(world);
                syscall(world, (), apply_deferred);
                react_to_all_removals_and_despawns(world);
            }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to an entity event.
/// - Returns number of callbacks queued.
fn react_to_entity_event_impl(
    rtype           : EntityReactType,
    component_id    : TypeId,
    commands        : &mut Commands,
    entity_reactors : &EntityReactors,
) -> usize
{
    // get cached callbacks
    let callbacks = match rtype
    {
        EntityReactType::Insertion => entity_reactors.insertion_callbacks.get(&component_id),
        EntityReactType::Mutation  => entity_reactors.mutation_callbacks.get(&component_id),
        EntityReactType::Removal   => entity_reactors.removal_callbacks.get(&component_id),
    };
    let Some(callbacks) = callbacks else { return 0; };

    // queue callbacks
    let mut callback_count = 0;
    for (_, cb) in callbacks
    {
        enque_reaction(commands, cb.clone());
        callback_count += 1;
    }

    callback_count
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to an entity event.
fn react_to_entity_event(
    In((rtype, entity, component_id)) : In<(EntityReactType, Entity, TypeId)>,
    mut commands                      : Commands,
    entity_reactors                   : Query<&EntityReactors>,
){
    // get this entity's entity reactors
    let Ok(entity_reactors) = entity_reactors.get(entity) else { return; };

    // react
    let _ = react_to_entity_event_impl(rtype, component_id, &mut commands, &entity_reactors);
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

/// React to tracked despawns.
/// - Returns number of callbacks queued.
//note: we cannot use RemovedComponents here because we need the ability to react to despawns that occur between
//      when 'register despawn tracker' is queued and executed
fn react_to_despawns_impl(
    mut commands    : Commands,
    mut react_cache : ResMut<ReactCache>,
) -> usize
{
    let mut callback_count = 0;

    while let Ok(despawned_entity) = react_cache.despawn_receiver.try_recv()
    {
        // remove prepared callbacks
        let Some(mut despawn_callbacks) = react_cache.despawn_reactors.remove(&despawned_entity) else { continue; };

        // queue despawn callbacks
        for (_, despawn_callback) in despawn_callbacks.drain(..)
        {
            enque_reaction(&mut commands, despawn_callback);
            callback_count += 1;
        }
    }

    callback_count
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

#[derive(Resource)]
struct ReactCache
{
    /// Callback id source. Used for reactor revocation.
    callback_counter: u64,

    /// query to get read-access to entity reactors
    entity_reactors_query: Option<QueryState<&'static EntityReactors>>,

    /// Per-component reactors
    component_reactors: HashMap<TypeId, ComponentReactors>,

    /// Components with removal reactors (cached to prevent duplicate insertion)
    tracked_removals: HashSet<TypeId>,
    /// Component removal checkers (as a vec for efficient iteration)
    removal_checkers: Vec<RemovalChecker>,
    /// Removal checker buffer (cached for reuse)
    removal_buffer: Option<Vec<Entity>>,

    // Entity despawn reactors
    //todo: is there a more efficient data structure? need faster cleanup on despawns
    despawn_reactors: HashMap<Entity, Vec<(u64, CallOnce<()>)>>,
    /// Despawn sender (cached for reuse with new despawn trackers)
    despawn_sender: crossbeam::channel::Sender<Entity>,
    /// Despawn receiver
    despawn_receiver: crossbeam::channel::Receiver<Entity>,

    /// Resource mutation reactors
    resource_reactors: HashMap<TypeId, Vec<(u64, Callback<()>)>>,
}

impl ReactCache
{
    fn next_callback_id(&mut self) -> u64
    {
        let counter = self.callback_counter;
        self.callback_counter += 1;
        counter
    }

    fn track_removals<C: Component>(&mut self)
    {
        // track removals of this component if untracked
        if self.tracked_removals.contains(&TypeId::of::<C>()) { return; };
        self.tracked_removals.insert(TypeId::of::<C>());
        self.removal_checkers.push(RemovalChecker::new::<C>());
    }

    fn register_insertion_reactor<C>(&mut self, callback: CallbackWith<(), Entity>) -> RevokeToken
    where
        C: Reactive +Send + Sync + 'static
    {
        let callback_id = self.next_callback_id();
        self.component_reactors
            .entry(TypeId::of::<C>())
            .or_default()
            .insertion_callbacks
            .push((callback_id, callback));

        RevokeToken{ reactor_type: ReactorType::ComponentInsertion(TypeId::of::<C>()), callback_id }
    }

    fn register_mutation_reactor<C>(&mut self, callback: CallbackWith<(), Entity>) -> RevokeToken
    where
        C: Reactive +Send + Sync + 'static
    {
        let callback_id = self.next_callback_id();
        self.component_reactors
            .entry(TypeId::of::<C>())
            .or_default()
            .mutation_callbacks
            .push((callback_id, callback));

        RevokeToken{ reactor_type: ReactorType::ComponentMutation(TypeId::of::<C>()), callback_id }
    }

    fn register_removal_reactor<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>) -> RevokeToken
    {
        let callback_id = self.next_callback_id();
        self.component_reactors
            .entry(TypeId::of::<C>())
            .or_default()
            .removal_callbacks
            .push((callback_id, callback));

        RevokeToken{ reactor_type: ReactorType::ComponentRemoval(TypeId::of::<C>()), callback_id }
    }

    fn register_despawn_reactor(&mut self, entity: Entity, callonce: CallOnce<()>) -> RevokeToken
    {
        let callback_id = self.next_callback_id();
        self.despawn_reactors
            .entry(entity)
            .or_default()
            .push((callback_id, callonce));

        RevokeToken{ reactor_type: ReactorType::Despawn(entity), callback_id }
    }

    fn register_resource_mutation_reactor<R: Send + Sync + 'static>(&mut self, callback: Callback<()>) -> RevokeToken
    {
        let callback_id = self.next_callback_id();
        self.resource_reactors
            .entry(TypeId::of::<R>())
            .or_default()
            .push((callback_id, callback));

        RevokeToken{ reactor_type: ReactorType::ResourceMutation(TypeId::of::<R>()), callback_id }
    }

    /// Revoke a component insertion reactor.
    fn revoke_component_reactor(&mut self, rtype: EntityReactType, comp_id: TypeId, callback_id: u64)
    {
        // get cached callbacks
        let Some(component_reactors) = self.component_reactors.get_mut(&comp_id) else { return; };
        let callbacks = match rtype
        {
            EntityReactType::Insertion => &mut component_reactors.insertion_callbacks,
            EntityReactType::Mutation  => &mut component_reactors.mutation_callbacks,
            EntityReactType::Removal   => &mut component_reactors.removal_callbacks
        };

        // revoke reactor
        for (idx, (id, _)) in callbacks.iter().enumerate()
        {
            if *id != callback_id { continue; }
            let _ = callbacks.remove(idx);  //todo: consider swap_remove()

            break;
        }

        // cleanup empty hashmap entries
        if !component_reactors.is_empty() { return; }
        let _ = self.component_reactors.remove(&comp_id);
    }

    /// Revoke a despawn reactor.
    fn revoke_despawn_reactor(&mut self, entity: Entity, callback_id: u64)
    {
        // get callbacks
        let Some(callbacks) = self.despawn_reactors.get_mut(&entity) else { return; };

        // revoke reactor
        for (idx, (id, _)) in callbacks.iter().enumerate()
        {
            if *id != callback_id { continue; }
            let _ = callbacks.remove(idx);  //todo: consider swap_remove()
            break;
        }

        // cleanup empty hashmap entries
        if callbacks.len() > 0 { return; }
        let _ = self.despawn_reactors.remove(&entity);
    }

    /// Revoke a resource mutation reactor.
    fn revoke_resource_mutation_reactor(&mut self, resource_id: TypeId, callback_id: u64)
    {
        // get callbacks
        let Some(callbacks) = self.resource_reactors.get_mut(&resource_id) else { return; };

        // revoke reactor
        for (idx, (id, _)) in callbacks.iter().enumerate()
        {
            if *id != callback_id { continue; }
            let _ = callbacks.remove(idx);  //todo: consider swap_remove()
            break;
        }

        // cleanup empty hashmap entries
        if callbacks.len() > 0 { return; }
        let _ = self.resource_reactors.remove(&resource_id);
    }

    /// Queue reactions to a component insertion.
    fn react_to_insertion<C: Reactive + Send + Sync + 'static>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity-specific component reactors
        commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Insertion, entity, TypeId::of::<C>()), react_to_entity_event)
            );

        // entity-agnostic component reactors
        let Some(handlers) = self.component_reactors.get(&TypeId::of::<C>()) else { return; };
        for (_, cb) in handlers.insertion_callbacks.iter()
        {
            enque_reaction(commands, cb.call_with(entity));
        }
    }

    /// Queue reactions to a component mutation.
    fn react_to_mutation<C: Reactive + Send + Sync + 'static>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity-specific component reactors
        commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Mutation, entity, TypeId::of::<C>()), react_to_entity_event)
            );

        // entity-agnostic component reactors
        let Some(handlers) = self.component_reactors.get(&TypeId::of::<C>()) else { return; };
        for (_, cb) in handlers.mutation_callbacks.iter()
        {
            enque_reaction(commands, cb.call_with(entity));
        }
    }

    /// React to component removals
    /// - Returns number of callbacks queued.
    /// - Note: We must use a command queue since the react cache is not present in the world, so callbacks may be invalid
    ///   until the react cache is re-inserted. The react cache is removed from the world so we can call removal checkers
    ///   directly (they are type-erased syscalls).
    fn react_to_removals(&mut self, world: &mut World, command_queue: &mut CommandQueue) -> usize
    {
        // extract cached
        let mut buffer = self.removal_buffer.take().unwrap_or_else(|| Vec::default());
        let mut query  = self.entity_reactors_query.take().unwrap_or_else(|| world.query::<&EntityReactors>());

        // process all removal checkers
        let mut callback_count = 0;

        for checker in &mut self.removal_checkers
        {
            // check for removals
            buffer = checker.checker.call(world, buffer);
            if buffer.len() == 0 { continue; }

            // queue removal callbacks
            let mut commands = Commands::new(command_queue, world);

            for entity in buffer.iter()
            {
                // ignore entities that don't exist
                if world.get_entity(*entity).is_none() { continue; }

                // entity-specific component reactors
                if let Ok(entity_reactors) = query.get(world, *entity)
                {
                    callback_count += react_to_entity_event_impl(
                            EntityReactType::Removal,
                            checker.component_id,
                            &mut commands,
                            &entity_reactors
                        );
                }

                // entity-agnostic component reactors
                let Some(reactors) = self.component_reactors.get(&checker.component_id) else { continue; };
                for (_, cb) in reactors.removal_callbacks.iter()
                {
                    enque_reaction(&mut commands, cb.call_with(*entity));
                    callback_count += 1;
                }
            }
        }

        // return cached
        self.removal_buffer        = Some(buffer);
        self.entity_reactors_query = Some(query);

        callback_count
    }

    /// Queue reactions to a resource mutation.
    fn react_to_resource_mutation<R: Send + Sync + 'static>(&mut self, commands: &mut Commands)
    {
        // resource handlers
        let Some(handlers) = self.resource_reactors.get(&TypeId::of::<R>()) else { return; };
        for (_, cb) in handlers.iter()
        {
            enque_reaction(commands, cb.clone());
        }
    }
}

impl Default for ReactCache
{
    fn default() -> Self
    {
        // prep despawn channel
        let (despawn_sender, despawn_receiver) = crossbeam::channel::unbounded::<Entity>();

        Self{
            callback_counter      : 0,
            entity_reactors_query : None,
            component_reactors    : HashMap::default(),
            tracked_removals      : HashSet::default(),
            removal_checkers      : Vec::new(),
            removal_buffer        : None,
            despawn_reactors      : HashMap::new(),
            despawn_sender,
            despawn_receiver,
            resource_reactors     : HashMap::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tag trait for identifying reactive objects.
pub trait Reactive {}

//-------------------------------------------------------------------------------------------------------------------

/// Component wrapper that enables reacting to component mutations.
/// - WARNING: It is possible to remove a `React` from one entity and manually insert it to another entity. That WILL
///            break the react framework. Instead use `react_commands.insert(new_entity, react_component.take());`.
#[derive(Component)]
pub struct React<C: Send + Sync + 'static>
{
    entity    : Entity,
    component : C,
}

impl<C: Send + Sync + 'static> React<C>
{
    /// Mutably access the component and trigger reactions.
    pub fn get_mut<'a>(&'a mut self, rcommands: &mut ReactCommands) -> &'a mut C
    {
        if let Some(ref mut cache) = rcommands.cache
        {
            cache.react_to_mutation::<React<C>>(&mut rcommands.commands, self.entity);
        }
        &mut self.component
    }

    /// Mutably access the component without triggering reactions.
    pub fn get_mut_noreact(&mut self) -> &mut C
    {
        &mut self.component
    }

    /// Unwrap the `React`.
    pub fn take(self) -> C
    {
        self.component
    }
}

impl<C: Send + Sync + 'static> Deref for React<C>
{
    type Target = C;

    fn deref(&self) -> &C
    {
        &self.component
    }
}

impl<C: Send + Sync + 'static> Reactive for React<C> {}

//-------------------------------------------------------------------------------------------------------------------

/// Resource wrapper that enables reacting to resource mutations.
#[derive(Resource)]
pub struct ReactRes<R: Send + Sync + 'static>
{
    resource: R,
}

impl<R: Send + Sync + 'static> ReactRes<R>
{
    /// New react resource.
    pub fn new(resource: R) -> Self
    {
        Self{ resource }
    }

    /// Mutably access the resource and trigger reactions.
    pub fn get_mut<'a>(&'a mut self, rcommands: &mut ReactCommands) -> &'a mut R
    {
        // note: we don't use `trigger_resource_mutation()` because that method panics if ReactPlugin was not added
        if let Some(ref mut cache) = rcommands.cache
        {
            cache.react_to_resource_mutation::<ReactRes<R>>(&mut rcommands.commands);
        }
        &mut self.resource
    }

    /// Mutably access the resource without triggering reactions.
    pub fn get_mut_noreact(&mut self) -> &mut R
    {
        &mut self.resource
    }

    /// Unwrap the `ReactRes`.
    pub fn take(self) -> R
    {
        self.resource
    }
}

impl<R: Send + Sync + 'static> Deref for ReactRes<R>
{
    type Target = R;

    fn deref(&self) -> &R
    {
        &self.resource
    }
}

impl<R: Send + Sync + 'static> Reactive for ReactRes<R> {}

//-------------------------------------------------------------------------------------------------------------------

/// Data sent to event reactors.
/// - Data can only be accessed immutably.
pub struct ReactEvent<E: Send + Sync + 'static>
{
    data: Arc<E>,
}

impl<E: Send + Sync + 'static> ReactEvent<E>
{
    pub fn new(data: E) -> Self
    {
        Self{ data: Arc::new(data) }
    }

    pub fn get(&self) -> &E { &self.data }
}

impl<E: Send + Sync + 'static> Clone for ReactEvent<E> { fn clone(&self) -> Self { Self{ data: self.data.clone() } } }
impl<E: Send + Sync + 'static> Reactive for ReactEvent<E> {}

//-------------------------------------------------------------------------------------------------------------------

/// React to component removals.
/// - Returns number of callbacks queued.
/// - If an entity has been despawned since the last time this was called, then any removals that occured in the meantime
///   will NOT be reacted to.
pub fn react_to_removals(world: &mut World) -> usize
{
    // remove cached
    let Some(mut react_cache) = world.remove_resource::<ReactCache>() else { return 0; };
    let mut command_queue = pop_react_command_queue(world);

    // process removals
    let callback_count = react_cache.react_to_removals(world, &mut command_queue);

    // return react cache
    world.insert_resource(react_cache);

    // apply commands
    command_queue.apply(world);

    // return command queue
    push_react_command_queue(world, command_queue);

    callback_count
}

//-------------------------------------------------------------------------------------------------------------------

/// React to tracked despawns.
/// - Returns number of callbacks queued.
pub fn react_to_despawns(world: &mut World) -> usize
{
    // check if we have a reactor
    if !world.contains_resource::<ReactCache>() { return 0; }

    // handle despawns
    syscall(world, (), react_to_despawns_impl)
}

//-------------------------------------------------------------------------------------------------------------------

/// Iteratively react to component removals and entity despawns until all reaction chains have ended.
pub fn react_to_all_removals_and_despawns(world: &mut World)
{
    // check if we have a reactor
    if !world.contains_resource::<ReactCache>() { return; }

    // loop until no more removals/despawns
    while syscall(world, (), react_to_removals) > 0 || syscall(world, (), react_to_despawns_impl) > 0 {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Eq, PartialEq, Debug)]
enum ReactorType
{
    EntityInsertion(Entity, TypeId),
    EntityMutation(Entity, TypeId),
    EntityRemoval(Entity, TypeId),
    ComponentInsertion(TypeId),
    ComponentMutation(TypeId),
    ComponentRemoval(TypeId),
    Despawn(Entity),
    ResourceMutation(TypeId),
}

/// Token for revoking reactors (event reactors use [`EventRevokeToken`]).
///
/// See [`ReactCommands::revoke()`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RevokeToken
{
    reactor_type : ReactorType,
    callback_id  : u64,
}

//-------------------------------------------------------------------------------------------------------------------

/// Token for revoking event reactors (non-event reactors use [`RevokeToken`]).
///
/// See [`ReactCommands::revoke_event_reactor()`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EventRevokeToken<E>
{
    callback_id : u64,
    _p          : PhantomData<E>
}

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
    commands : Commands<'w, 's>,
    cache    : Option<ResMut<'w, ReactCache>>,
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
        let notifier =  cache.despawn_sender.clone();
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

/// Prepares react framework so that reactors may be registered with [`ReactCommands`].
/// - Does NOT schedule any component removal or entity despawn reactor systems. You must schedule those yourself!
/// 
/// WARNING: If reactivity is implemented natively in Bevy, then this implementation will become obsolete.
#[bevy_plugin]
pub fn ReactPlugin(app: &mut App)
{
    app.init_resource::<ReactCache>();
}

//-------------------------------------------------------------------------------------------------------------------
