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
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Queue a react callback, followed by a call to `apply_deferred`.
/// - We want to apply deferred immediately so any side effects or chained reactions will be applied before any other
///   reactions.
fn enque_react_callback(commands: &mut Commands, cb: impl Command)
{
    commands.add(
            move |world: &mut World|
            {
                cb.apply(world);
                syscall(world, (), apply_deferred);  //todo: this may be quite slow, require caller to do it?
            }
        );
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
struct EntityReactHandlers
{
    insertion_callbacks : HashMap<TypeId, Vec<Callback<()>>>,
    mutation_callbacks  : HashMap<TypeId, Vec<Callback<()>>>,
    removal_callbacks   : HashMap<TypeId, Vec<Callback<()>>>,
}

impl Default for EntityReactHandlers
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

/// Add reaction to an entity. The reaction will be invoked when the event occurs on the entity.
fn add_reaction_to_entity<C: Send + Sync + 'static>(
    In((rtype, entity, callback)) : In<(EntityReactType, Entity, Callback<()>)>,
    mut commands                  : Commands,
    mut react_entities            : Query<&mut EntityReactHandlers>,
){
    // callback adder
    let add_callback =
        move |react_handlers: &mut EntityReactHandlers|
        {
            let callbacks = match rtype
            {
                EntityReactType::Insertion => react_handlers.insertion_callbacks.entry(TypeId::of::<C>()).or_default(),
                EntityReactType::Mutation  => react_handlers.mutation_callbacks.entry(TypeId::of::<C>()).or_default(),
                EntityReactType::Removal   => react_handlers.removal_callbacks.entry(TypeId::of::<C>()).or_default(),
            };
            callbacks.push(callback);
        };

    // add callback to entity
    match react_entities.get_mut(entity)
    {
        Ok(mut react_handlers) => add_callback(&mut react_handlers),
        _ =>
        {
            let Some(mut entity_commands) = commands.get_entity(entity) else { return; };

            // make new react handlers for the entity
            let mut react_handlers = EntityReactHandlers::default();

            // add callback and insert to entity
            add_callback(&mut react_handlers);
            entity_commands.insert(react_handlers);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to an entity event.
/// - Returns number of callbacks queued.
fn react_to_entity_event_impl(
    rtype           : EntityReactType,
    component_id    : TypeId,
    commands        : &mut Commands,
    react_handlers  : &EntityReactHandlers,
) -> usize
{
    // get cached callbacks
    let callbacks = match rtype
    {
        EntityReactType::Insertion => react_handlers.insertion_callbacks.get(&component_id),
        EntityReactType::Mutation  => react_handlers.mutation_callbacks.get(&component_id),
        EntityReactType::Removal   => react_handlers.removal_callbacks.get(&component_id),
    };
    let Some(callbacks) = callbacks else { return 0; };

    // queue callbacks
    let mut callback_count = 0;
    for cb in callbacks
    {
        enque_react_callback(commands, cb.clone());
        callback_count += 1;
    }

    callback_count
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to an entity event.
fn react_to_entity_event<C: Send + Sync + 'static>(
    In((rtype, entity)) : In<(EntityReactType, Entity)>,
    mut commands        : Commands,
    react_entities      : Query<&EntityReactHandlers>,
){
    // get this entity's react handlers
    let Ok(react_handlers) = react_entities.get(entity) else { return; };

    // finish react handling
    let _ = react_to_entity_event_impl(rtype, TypeId::of::<C>(), &mut commands, &react_handlers);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ComponentReactHandlers
{
    insertion_callbacks : Vec<CallbackWith<(), Entity>>,
    mutation_callbacks  : Vec<CallbackWith<(), Entity>>,
    removal_callbacks   : Vec<CallbackWith<(), Entity>>,
}

impl Default for ComponentReactHandlers
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

/// Note: `RemovedComponents` acts like an event reader, so multiple invocations of this system within one tick will
/// not see duplicate removals.
fn check_component_removals<C: Send + Sync + 'static>(
    In(mut buffer) : In<Vec<Entity>>,
    mut removed    : RemovedComponents<React<C>>
) -> Vec<Entity>
{
    buffer.clear();
    removed.iter().for_each(|entity| buffer.push(entity));
    buffer
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
    fn new<C: Send + Sync + 'static>() -> Self
    {
        Self{
            component_id : TypeId::of::<C>(),
            checker      : SysCall::new(|world, buffer| syscall(world, buffer, check_component_removals::<C>)),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tag for tracking despawns of entities with despawn reactors.
#[derive(Component)]
struct DespawnTracker;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to tracked despawns.
/// - Returns number of callbacks queued.
fn react_to_despawns_impl(
    mut commands    : Commands,
    mut react_cache : ResMut<ReactCache>,
    despawn_tracked : Query<Entity, With<DespawnTracker>>,
) -> usize
{
    let mut callback_count = 0;
    react_cache.despawn_handlers.retain(
            |entity, despawn_callbacks|
            {
                // keep if entity is alive
                if despawn_tracked.contains(*entity) { return true; }

                // queue despawn callbacks
                for despawn_callback in despawn_callbacks.drain(..)
                {
                    enque_react_callback(&mut commands, despawn_callback);
                    callback_count += 1;
                }
                false
            }
        );

    callback_count
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct ReactCache
{
    /// query to get read-access to entity handlers
    entity_handler_query: Option<QueryState<&'static EntityReactHandlers>>,

    /// Per-component handlers
    component_handlers: HashMap<TypeId, ComponentReactHandlers>,

    /// Components with removal reactors
    tracked_removals: HashSet<TypeId>,
    /// Component removal checkers
    removal_checkers: Vec<RemovalChecker>,
    /// Removal checker buffer (cached for reuse)
    removal_buffer: Option<Vec<Entity>>,

    // Entity despawn handlers
    //todo: is there a more efficient data structure? need faster cleanup on despawns
    despawn_handlers: HashMap<Entity, Vec<CallOnce<()>>>,

    /// Resource mutation handlers
    resource_handlers: HashMap<TypeId, Vec<Callback<()>>>,
}

impl ReactCache
{
    fn track_removals<C: Send + Sync + 'static>(&mut self)
    {
        // track removals of this component if untracked
        if self.tracked_removals.contains(&TypeId::of::<C>()) { return; };
        self.tracked_removals.insert(TypeId::of::<C>());
        self.removal_checkers.push(RemovalChecker::new::<C>());
    }

    fn register_insertion_reaction<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>)
    {
        self.component_handlers
            .entry(TypeId::of::<C>())
            .or_default()
            .insertion_callbacks
            .push(callback);
    }

    fn register_mutation_reaction<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>)
    {
        self.component_handlers
            .entry(TypeId::of::<C>())
            .or_default()
            .mutation_callbacks
            .push(callback);
    }

    fn register_removal_reaction<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>)
    {
        self.component_handlers
            .entry(TypeId::of::<C>())
            .or_default()
            .removal_callbacks
            .push(callback);
    }

    fn register_despawn_reaction(&mut self, entity: Entity, callonce: CallOnce<()>)
    {
        self.despawn_handlers
            .entry(entity)
            .or_default()
            .push(callonce);
    }

    fn register_resource_mutation_reaction<R: Send + Sync + 'static>(&mut self, callback: Callback<()>)
    {
        self.resource_handlers
            .entry(TypeId::of::<R>())
            .or_default()
            .push(callback);
    }

    /// Queue reactions to a component insertion.
    /// - Initializes removal checkers for newly-encountered component types. We only need to do that here because
    ///   all entities with `React` components acquired those components from a `ReactCommand::insert()` which uses
    ///   this method.
    fn react_on_insert<C: Send + Sync + 'static>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity handlers
        commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Insertion, entity), react_to_entity_event::<C>)
            );

        // queue component insertion callbacks
        let Some(handlers) = self.component_handlers.get(&TypeId::of::<C>()) else { return; };
        for cb in handlers.insertion_callbacks.iter()
        {
            enque_react_callback(commands, cb.call_with(entity));
        }
    }

    /// Queue reactions to a component mutation.
    fn react_on_mutation<C: Send + Sync + 'static>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity handlers
        commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Mutation, entity), react_to_entity_event::<C>)
            );

        // component handlers
        let Some(handlers) = self.component_handlers.get(&TypeId::of::<C>()) else { return; };
        for cb in handlers.mutation_callbacks.iter()
        {
            enque_react_callback(commands, cb.call_with(entity));
        }
    }

    /// Queue reactions to a resource mutation.
    fn react_on_resource_mutation<R: Send + Sync + 'static>(&mut self, commands: &mut Commands)
    {
        // resource handlers
        let Some(handlers) = self.resource_handlers.get(&TypeId::of::<R>()) else { return; };
        for cb in handlers.iter()
        {
            enque_react_callback(commands, cb.clone());
        }
    }

    /// React to component removals
    /// - Returns number of callbacks queued.
    /// - We must use a command queue since the react cache is not present in the world, so callbacks may be invalid
    ///   until the react cache is re-inserted.
    fn react_to_removals(&mut self, world: &mut World, command_queue: &mut CommandQueue) -> usize
    {
        // extract cached
        let mut buffer = self.removal_buffer.take().unwrap_or_else(|| Vec::default());
        let mut query  = self.entity_handler_query.take().unwrap_or_else(|| world.query::<&EntityReactHandlers>());

        // process all removal checkers
        let mut callback_count = 0;

        for checker in &mut self.removal_checkers
        {
            // check for removals
            buffer = checker.checker.call(world, buffer);

            // queue removal callbacks
            let mut commands = Commands::new(command_queue, world);

            for entity in buffer.iter()
            {
                // entity handlers
                if let Ok(react_handlers) = query.get(world, *entity)
                {
                    callback_count += react_to_entity_event_impl(
                            EntityReactType::Removal,
                            checker.component_id,
                            &mut commands,
                            &react_handlers
                        );
                }

                // component handlers
                let Some(handlers) = self.component_handlers.get(&checker.component_id) else { continue; };
                for cb in handlers.removal_callbacks.iter()
                {
                    enque_react_callback(&mut commands, cb.call_with(*entity));
                    callback_count += 1;
                }
            }
        }

        // return cached
        self.removal_buffer       = Some(buffer);
        self.entity_handler_query = Some(query);

        callback_count
    }
}

impl Default for ReactCache
{
    fn default() -> Self
    {
        Self{
            entity_handler_query : None,
            component_handlers   : HashMap::default(),
            tracked_removals     : HashSet::default(),
            removal_checkers     : Vec::new(),
            removal_buffer       : None,
            despawn_handlers     : HashMap::new(),
            resource_handlers    : HashMap::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ReactCommandQueue(Vec<CommandQueue>);

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Drives reactivity.
/// - Wraps `Commands`. Use `.commands()` to access it if needed.
#[derive(SystemParam)]
pub struct ReactCommands<'w, 's>
{
    commands : Commands<'w, 's>,
    cache    : ResMut<'w, ReactCache>,
}

impl<'w, 's> ReactCommands<'w, 's>
{
    /// Access `Commands`.
    pub fn commands<'a>(&'a mut self) -> &'a mut Commands<'w, 's>
    {
        &mut self.commands
    }

    /// Insert a `React<C>` to the specified entity.
    /// - Does nothing if the entity does not exist.
    //todo: consider more ergonomic entity access, e.g. ReactEntityCommands
    pub fn insert<'a, C: Send + Sync + 'static>(&'a mut self, entity: Entity, component: C)
    {
        let Some(mut entity_commands) = self.commands.get_entity(entity) else { return; };
        entity_commands.insert( React{ entity, component } );
        self.cache.react_on_insert::<C>(&mut self.commands, entity);
    }

    /// React when a `React<C>` is inserted on a specific entity.
    /// - Does nothing if the entity does not exist.
    pub fn react_to_insertion_on_entity<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity   : Entity,
        callback : impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Insertion, entity, Callback::new(callback)), add_reaction_to_entity::<C>)
            );
    }

    /// React when a `React<C>` is inserted on any entity.
    /// - Takes the entity the component was inserted to.
    pub fn react_to_insertion_on_any<'a, C: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ){
        self.cache.register_insertion_reaction::<C>(CallbackWith::new(callback));
    }

    /// React when a `React<C>` is mutated on a specific entity.
    /// - Does nothing if the entity does not exist.
    pub fn react_to_mutation_on_entity<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity   : Entity,
        callback : impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Mutation, entity, Callback::new(callback)), add_reaction_to_entity::<C>)
            );
    }

    /// React when a `React<C>` is mutated on any entity.
    /// - Takes the entity the component was mutated on.
    pub fn react_to_mutation_on_any<'a, C: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ){
        self.cache.register_mutation_reaction::<C>(CallbackWith::new(callback));
    }

    /// React when a `React<C>` is removed from a specific entity.
    /// - Does nothing if the entity does not exist.
    pub fn react_to_removal_from_entity<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity   : Entity,
        callback : impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.cache.track_removals::<C>();
        self.commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Removal, entity, Callback::new(callback)), add_reaction_to_entity::<C>)
            );
    }

    /// React when a `React<C>` is removed from any entity.
    /// - Takes the entity the component was removed from.
    pub fn react_to_removal_from_any<'a, C: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ){
        self.cache.track_removals::<C>();
        self.cache.register_removal_reaction::<C>(CallbackWith::new(callback));
    }

    /// React when an entity is despawned.
    /// - Does nothing if the entity does not exist.
    pub fn react_to_despawn<'a>(
        &'a mut self,
        entity: Entity,
        callonce: impl FnOnce(&mut World) -> () + Send + Sync + 'static
    ){
        let Some(mut entity_commands) = self.commands.get_entity(entity) else { return; };
        entity_commands.insert( DespawnTracker );
        self.cache.register_despawn_reaction(entity, CallOnce::new(callonce));
    }

    /// React when a resource is mutated.
    /// - Reactions only occur for `ReactRes<R>` resources accessed with `ReactRes::get_mut()`.
    pub fn react_to_resource_mutation<'a, R: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.cache.register_resource_mutation_reaction::<R>(Callback::new(callback));
    }
}

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
    pub fn get_mut<'a>(&'a mut self, react_commands: &mut ReactCommands) -> &'a mut C
    {
        react_commands.cache.react_on_mutation::<C>(&mut react_commands.commands, self.entity);
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
    pub fn get_mut<'a>(&'a mut self, react_commands: &mut ReactCommands) -> &'a mut R
    {
        react_commands.cache.react_on_resource_mutation::<R>(&mut react_commands.commands);
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

//-------------------------------------------------------------------------------------------------------------------

/// React to component removals.
/// - Returns number of callbacks queued.
/// - If an entity has been despawned since the last time this was called, then any removals that occured in the meantime
///   will NOT be reacted to.
pub fn react_to_removals(world: &mut World) -> usize
{
    // remove cached
    let Some(mut react_cache) = world.remove_resource::<ReactCache>() else { return 0; };
    let mut command_queue = world
        .get_resource_or_insert_with(|| ReactCommandQueue::default())
        .0
        .pop()
        .unwrap_or_else(|| CommandQueue::default());

    // process removals
    let callback_count = react_cache.react_to_removals(world, &mut command_queue);

    // return react cache
    world.insert_resource(react_cache);

    // apply commands
    command_queue.apply(world);

    // return command queue
    // - note: we use a container of command queues in case of recursion
    world.get_resource_or_insert_with(|| ReactCommandQueue::default()).0.push(command_queue);

    callback_count
}

//-------------------------------------------------------------------------------------------------------------------

/// React to tracked despawns.
/// - Returns number of callbacks queued.
pub fn react_to_despawns(world: &mut World) -> usize
{
    syscall(world, (), react_to_despawns_impl)
}

//-------------------------------------------------------------------------------------------------------------------

/// Iteratively react to component removals and entity despawns until all reaction chains have ended.
/// - WARNING: This may be inefficient if you are reacting to many component removals and entity despawns.
pub fn react_to_all_removals_and_despawns(world: &mut World)
{
    // check if we have a reactor
    if !world.contains_resource::<ReactCache>() { return; }

    // loop until no more removals/despawns
    while syscall(world, (), react_to_removals) > 0 || syscall(world, (), react_to_despawns_impl) > 0 {}
}

//-------------------------------------------------------------------------------------------------------------------

/// Prepares a reactor core so that `ReactCommands` may be accessed in systems.
/// - Does NOT schedule any component removal/entity despawn reactor systems. You must schedule those yourself!
#[bevy_plugin]
pub fn ReactPlugin(app: &mut App)
{
    app.init_resource::<ReactCache>();
}

//-------------------------------------------------------------------------------------------------------------------
