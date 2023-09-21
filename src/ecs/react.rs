//local shortcuts
use crate::ecs::*;

//third-party shortcuts
use bevy::ecs::system::{CommandQueue, SystemParam};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_fn_plugin::*;

//standard shortcuts
use core::any::TypeId;
use core::ops::Deref;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Note: `RemovedComponents` acts like an event reader, so multiple invocations of this system within one tick will
/// not see duplicate removals.
fn check_component_removal<C: Send + Sync + 'static>(
    In(mut buffer): In<Vec<Entity>>,
    mut removed: RemovedComponents<React<C>>
) -> Vec<Entity>
{
    buffer.clear();
    removed.iter().for_each(|entity| buffer.push(entity));
    buffer
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ReactorEntityHandlers
{
    insertion_callbacks : HashMap<TypeId, Vec<Callback<()>>>,
    mutation_callbacks  : HashMap<TypeId, Vec<Callback<()>>>,
    removal_callbacks   : HashMap<TypeId, Vec<Callback<()>>>,
    despawn_callbacks   : Vec<Callback<()>>,
}

impl Default for ReactorEntityHandlers
{
    fn default() -> Self
    {
        Self{
            insertion_callbacks : HashMap::new(),
            mutation_callbacks  : HashMap::new(),
            removal_callbacks   : HashMap::new(),
            despawn_callbacks   : Vec::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ReactorComponentHandlers
{
    insertion_callbacks : Vec<CallbackWith<(), Entity>>,
    mutation_callbacks  : Vec<CallbackWith<(), Entity>>,
    removal_callbacks   : Vec<CallbackWith<(), Entity>>,
}

impl Default for ReactorComponentHandlers
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
    buffer       : Option<Vec<Entity>>,
    checker      : SysCall<(), Vec<Entity>, Vec<Entity>>
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct ReactorCore
{
    /// Per-entity handlers
    ///- We track despawns here instead of a separate map because we need to iterate this map to clean up despawns.
    entity_handlers: HashMap<Entity, ReactorEntityHandlers>,

    /// Per-component handlers
    component_handlers: HashMap<TypeId, ReactorComponentHandlers>,

    /// Component removal checkers
    removal_checkers: Vec<RemovalChecker>,
}

impl ReactorCore
{
    /// Queue reactions to a component insertion.
    fn react_on_insert<C: Send + Sync + 'static>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity handlers
        let handler = self.entity_handlers.entry(entity).or_default();
        for cb in handler.insertion_callbacks.entry(TypeId::of::<C>()).or_default()
        {
            commands.add(cb.clone());
        }

        // add removal checker if this component is unknown
        let mut handler = self.component_handlers.entry(TypeId::of::<C>());

        if let bevy::utils::hashbrown::hash_map::Entry::Vacant(_) = &mut handler
        {
            self.removal_checkers.push(RemovalChecker{
                        component_id : TypeId::of::<C>(),
                        buffer       : Some(Vec::default()),
                        checker      : SysCall::new(|world, buffer| syscall(world, buffer, check_component_removal::<C>))
                    }
                );
        }

        // component handlers
        for cb in handler.or_default().insertion_callbacks.iter()
        {
            commands.add(cb.call_with(entity));
        }
    }

    /// Queue reactions to a component mutation.
    fn react_on_mutation<C: Send + Sync + 'static>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity handlers
        let handler = self.entity_handlers.entry(entity).or_default();
        for cb in handler.mutation_callbacks.entry(TypeId::of::<C>()).or_default()
        {
            commands.add(cb.clone());
        }

        // component handlers
        for cb in self.component_handlers.entry(TypeId::of::<C>()).or_default().mutation_callbacks.iter()
        {
            commands.add(cb.call_with(entity));
        }
    }

    fn register_insertion_reaction<C: Send + Sync + 'static>(&mut self, entity: Entity, callback: Callback<()>)
    {
        self.entity_handlers
            .entry(entity)
            .or_default()
            .insertion_callbacks
            .entry(TypeId::of::<C>())
            .or_default()
            .push(callback);
    }

    fn register_insertion_reaction_any<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>)
    {
        self.component_handlers
            .entry(TypeId::of::<C>())
            .or_default()
            .insertion_callbacks
            .push(callback);
    }

    fn register_mutation_reaction<C: Send + Sync + 'static>(&mut self, entity: Entity, callback: Callback<()>)
    {
        self.entity_handlers
            .entry(entity)
            .or_default()
            .mutation_callbacks
            .entry(TypeId::of::<C>())
            .or_default()
            .push(callback);
    }

    fn register_mutation_reaction_any<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>)
    {
        self.component_handlers
            .entry(TypeId::of::<C>())
            .or_default()
            .mutation_callbacks
            .push(callback);
    }

    fn register_removal_reaction<C: Send + Sync + 'static>(&mut self, entity: Entity, callback: Callback<()>)
    {
        self.entity_handlers
            .entry(entity)
            .or_default()
            .removal_callbacks
            .entry(TypeId::of::<C>())
            .or_default()
            .push(callback);
    }

    fn register_removal_reaction_any<C: Send + Sync + 'static>(&mut self, callback: CallbackWith<(), Entity>)
    {
        self.component_handlers
            .entry(TypeId::of::<C>())
            .or_default()
            .removal_callbacks
            .push(callback);
    }

    fn register_despawn_reaction(&mut self, entity: Entity, callback: Callback<()>)
    {
        self.entity_handlers
            .entry(entity)
            .or_default()
            .despawn_callbacks
            .push(callback);
    }
}

impl Default for ReactorCore
{
    fn default() -> Self
    {
        Self{
            entity_handlers    : HashMap::default(),
            component_handlers : HashMap::default(),
            removal_checkers   : Vec::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// React to tracked despawns.
/// - Returns number of callbacks queued.
fn react_to_despawns_impl(
    mut commands     : Commands,
    mut reactor_core : ResMut<ReactorCore>,
    despawn_tracked  : Query<Entity, With<DespawnTracker>>
) -> usize
{
    let mut callback_count = 0;
    reactor_core.entity_handlers.retain(
            |entity, entity_handlers|
            {
                // keep if entity is alive
                if despawn_tracked.contains(*entity) { return true; }

                // queue despawn callbacks
                for despawn_callback in entity_handlers.despawn_callbacks.drain(..)
                { commands.add(despawn_callback); callback_count += 1; }
                false
            }
        );

    callback_count
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ReactorCommandQueue
{
    queue: CommandQueue,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tag for tracking despawns of reactable entities.
#[derive(Component)]
struct DespawnTracker;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Drives reactivity.
/// - Wraps `Commands`. Use `.commands()` to access it if needed.
#[derive(SystemParam)]
pub struct ReactCommands<'w, 's>
{
    commands : Commands<'w, 's>,
    reactor  : ResMut<'w, ReactorCore>,
}

impl<'w, 's> ReactCommands<'w, 's>
{
    /// Access `Commands`.
    pub fn commands<'a>(&'a mut self) -> &'a mut Commands<'w, 's>
    {
        &mut self.commands
    }

    /// Insert a `React<C>` to the specified entity.
    /// - Spawns the entity if it doesn't exist.
    //todo: consider more ergonomic entity access, e.g. ReactEntityCommands
    pub fn insert<'a, C: Send + Sync + 'static>(&'a mut self, entity: Entity, component: C)
    {
        self.commands.get_or_spawn(entity).insert( (React{ entity, component }, DespawnTracker) );
        self.reactor.react_on_insert::<C>(&mut self.commands, entity);
    }

    /// React when a `React<C>` is inserted on a specific entity.
    /// - Spawns the entity if it doesn't exist.
    pub fn react_to_insert<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity   : Entity,
        callback : impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.commands.get_or_spawn(entity).insert(DespawnTracker);
        self.reactor.register_insertion_reaction::<C>(entity, Callback::new(callback));
    }

    /// React when a `React<C>` is inserted on any entity.
    /// - Takes the entity the component was inserted to.
    pub fn react_to_insert_on_any<'a, C: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ){
        self.reactor.register_insertion_reaction_any::<C>(CallbackWith::new(callback));
    }

    /// React when a `React<C>` is mutated on a specific entity.
    /// - Spawns the entity if it doesn't exist.
    pub fn react_to_mutation<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity   : Entity,
        callback : impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.commands.get_or_spawn(entity).insert(DespawnTracker);
        self.reactor.register_mutation_reaction::<C>(entity, Callback::new(callback));
    }

    /// React when a `React<C>` is mutated on any entity.
    /// - Takes the entity the component was mutated on.
    pub fn react_to_mutation_on_any<'a, C: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ){
        self.reactor.register_mutation_reaction_any::<C>(CallbackWith::new(callback));
    }

    /// React when a `React<C>` is removed from a specific entity.
    /// - Spawns the entity if it doesn't exist.
    pub fn react_to_removal<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity   : Entity,
        callback : impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.commands.get_or_spawn(entity).insert(DespawnTracker);
        self.reactor.register_removal_reaction::<C>(entity, Callback::new(callback));
    }

    /// React when a `React<C>` is removed from any entity.
    /// - Takes the entity the component was removed from.
    pub fn react_to_removal_from_any<'a, C: Send + Sync + 'static>(
        &'a mut self,
        callback: impl Fn(&mut World, Entity) -> () + Send + Sync + 'static
    ){
        self.reactor.register_removal_reaction_any::<C>(CallbackWith::new(callback));
    }

    /// React when an entity is despawned.
    /// - Spawns the entity if it doesn't exist.
    pub fn react_to_despawn<'a, C: Send + Sync + 'static>(
        &'a mut self,
        entity: Entity,
        callback: impl Fn(&mut World) -> () + Send + Sync + 'static
    ){
        self.commands.get_or_spawn(entity).insert(DespawnTracker);
        self.reactor.register_despawn_reaction(entity, Callback::new(callback));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component wrapper that facilitates reacting to component mutations.
/// - WARNING: It is possible to remove a `React` from one entity and manually insert it to another entity. That WILL
///            break the reactor core. Instead use `react_commands.insert(new_entity, react_component.take())`.
#[derive(Component)]
pub struct React<C: Send + Sync + 'static>
{
    entity    : Entity,
    component : C,
}

impl<C: Send + Sync + 'static> React<C>
{
    pub fn get_mut<'a>(&'a mut self, engine: &mut ReactCommands) -> &'a mut C
    {
        engine.reactor.react_on_mutation::<C>(&mut engine.commands, self.entity);
        &mut self.component
    }

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

/// React to component removals.
/// - Returns number of callbacks queued.
pub fn react_to_removals(world: &mut World) -> usize
{
    // extract reactor core
    let Some(mut reactor) = world.remove_resource::<ReactorCore>() else { return 0; };
    let mut command_queue = world.remove_resource::<ReactorCommandQueue>().unwrap_or(ReactorCommandQueue::default());

    // process all removal checkers
    let mut callback_count = 0;

    for checker in &mut reactor.removal_checkers
    {
        // check for removals
        let buffer = checker.buffer.take().unwrap_or(Vec::default());
        let buffer = checker.checker.call(world, buffer);

        // queue removal callbacks
        for entity in buffer.iter()
        {
            // entity handlers
            let handler = reactor.entity_handlers.entry(*entity).or_default();
            for cb in handler.removal_callbacks.entry(checker.component_id).or_default()
            {
                command_queue.queue.push(cb.clone());
                callback_count += 1;
            }

            // component handlers
            for cb in reactor.component_handlers.entry(checker.component_id).or_default().removal_callbacks.iter()
            {
                command_queue.queue.push(cb.call_with(*entity));
                callback_count += 1;
            }
        }
        checker.buffer = Some(buffer);
    }

    // reinsert the reactor core
    world.insert_resource(reactor);

    // apply any queued callbacks
    command_queue.queue.apply(world);
    world.insert_resource(command_queue);

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
/// - WARNING: This may be inefficient if you are reacting to many entities and components.
pub fn react_to_all_removals_and_despawns(world: &mut World)
{
    // check if we have a reactor
    if !world.contains_resource::<ReactorCore>() { return; }

    // loop until no more removals/despawns
    while syscall(world, (), react_to_removals) > 0 || syscall(world, (), react_to_despawns_impl) > 0 {}
}

//-------------------------------------------------------------------------------------------------------------------

/// Prepares a reactor core so that `ReactCommands` may be accessed in systems.
/// - Does NOT schedule any component removal/entity despawn reactor systems. You must schedule those yourself!
#[bevy_plugin]
pub fn ReactPlugin(app: &mut App)
{
    app.init_resource::<ReactorCore>();
}

//-------------------------------------------------------------------------------------------------------------------
