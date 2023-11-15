//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

//standard shortcuts
use core::any::TypeId;
use std::vec::Vec;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ComponentReactors
{
    insertion_callbacks : Vec<SysId>,
    mutation_callbacks  : Vec<SysId>,
    removal_callbacks   : Vec<SysId>,
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
    fn new<C: ReactComponent>() -> Self
    {
        Self{
            component_id : TypeId::of::<C>(),
            checker      : SysCall::new(|world, buffer| syscall(world, buffer, collect_component_removals::<C>)),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Collect component removals.
///
/// Note: `RemovedComponents` acts like an event reader, so multiple invocations of this system within one tick will
/// not see duplicate removals.
fn collect_component_removals<C: ReactComponent>(
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
    for sys_id in callbacks
    {
        enque_reaction(commands, *sys_id, ());
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

#[derive(Resource)]
pub(crate) struct ReactCache
{
    /// Callback id source. Used for reactor revocation and ensuring reactors have unique Local state.
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
    resource_reactors: HashMap<TypeId, Vec<SysId>>,

    /// Data event reactors
    event_reactors: HashMap<TypeId, Vec<SysId>>,
}

impl ReactCache
{
    pub(crate) fn next_callback_id(&mut self) -> u64
    {
        let counter = self.callback_counter;
        self.callback_counter += 1;
        counter
    }

    pub(crate) fn despawn_sender(&self) -> crossbeam::channel::Sender<Entity>
    {
        self.despawn_sender.clone()
    }

    pub(crate) fn try_recv_despawn(&self) -> Option<Entity>
    {
        self.despawn_receiver.try_recv().ok()
    }

    pub(crate) fn remove_despawn_reactors(&mut self, despawned_entity: Entity) -> Option<Vec<(u64, CallOnce<()>)>>
    {
        self.despawn_reactors.remove(&despawned_entity)
    }

    pub(crate) fn track_removals<C: ReactComponent>(&mut self)
    {
        // track removals of this component if untracked
        if self.tracked_removals.contains(&TypeId::of::<C>()) { return; };
        self.tracked_removals.insert(TypeId::of::<C>());
        self.removal_checkers.push(RemovalChecker::new::<C>());
    }

    pub(crate) fn register_insertion_reactor<C: ReactComponent>(&mut self, sys_id: SysId) -> ReactorType
    {
        self.component_reactors
            .entry(TypeId::of::<C>())
            .or_default()
            .insertion_callbacks
            .push(sys_id);

        ReactorType::ComponentInsertion(TypeId::of::<C>())
    }

    pub(crate) fn register_mutation_reactor<C: ReactComponent>(&mut self, sys_id: SysId) -> ReactorType
    {
        self.component_reactors
            .entry(TypeId::of::<C>())
            .or_default()
            .mutation_callbacks
            .push(sys_id);

        ReactorType::ComponentMutation(TypeId::of::<C>())
    }

    pub(crate) fn register_removal_reactor<C: ReactComponent>(&mut self, sys_id: SysId) -> ReactorType
    {
        self.component_reactors
            .entry(TypeId::of::<C>())
            .or_default()
            .removal_callbacks
            .push(sys_id);

        ReactorType::ComponentRemoval(TypeId::of::<C>())
    }

    pub(crate) fn register_resource_mutation_reactor<R: ReactResource>(&mut self, sys_id: SysId) -> ReactorType
    {
        self.resource_reactors
            .entry(TypeId::of::<R>())
            .or_default()
            .push(sys_id);

        ReactorType::ResourceMutation(TypeId::of::<R>())
    }

    pub(crate) fn register_event_reactor<E: 'static>(&mut self, sys_id: SysId) -> ReactorType
    {
        self.event_reactors
            .entry(TypeId::of::<E>())
            .or_default()
            .push(sys_id);

        ReactorType::Event(TypeId::of::<E>())
    }

    pub(crate) fn register_despawn_reactor(&mut self, entity: Entity, callonce: CallOnce<()>) -> RevokeToken
    {
        let callback_id = self.next_callback_id();
        self.despawn_reactors
            .entry(entity)
            .or_default()
            .push((callback_id, callonce));

        RevokeToken{ reactors: vec![ReactorType::Despawn(entity)], sys_id: SysId::new_raw::<ReactCallback<()>>(callback_id) }
    }

    /// Revoke a component insertion reactor.
    pub(crate) fn revoke_component_reactor(&mut self, rtype: EntityReactType, comp_id: TypeId, sys_id: SysId)
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
        for (idx, id) in callbacks.iter().enumerate()
        {
            if *id != sys_id { continue; }
            let _ = callbacks.remove(idx);  //todo: consider swap_remove()

            break;
        }

        // cleanup empty hashmap entries
        if !component_reactors.is_empty() { return; }
        let _ = self.component_reactors.remove(&comp_id);
    }

    /// Revoke a resource mutation reactor.
    pub(crate) fn revoke_resource_mutation_reactor(&mut self, resource_id: TypeId, sys_id: SysId)
    {
        // get callbacks
        let Some(callbacks) = self.resource_reactors.get_mut(&resource_id) else { return; };

        // revoke reactor
        for (idx, id) in callbacks.iter().enumerate()
        {
            if *id != sys_id { continue; }
            let _ = callbacks.remove(idx);  //todo: consider swap_remove()
            break;
        }

        // cleanup empty hashmap entries
        if callbacks.len() > 0 { return; }
        let _ = self.resource_reactors.remove(&resource_id);
    }

    /// Revoke an event reactor.
    pub(crate) fn revoke_event_reactor(&mut self, event_id: TypeId, sys_id: SysId)
    {
        // get callbacks
        let Some(callbacks) = self.event_reactors.get_mut(&event_id) else { return; };

        // revoke reactor
        for (idx, id) in callbacks.iter().enumerate()
        {
            if *id != sys_id { continue; }
            let _ = callbacks.remove(idx);  //todo: consider swap_remove()
            break;
        }

        // cleanup empty hashmap entries
        if callbacks.len() > 0 { return; }
        let _ = self.event_reactors.remove(&event_id);
    }

    /// Revoke a despawn reactor.
    pub(crate) fn revoke_despawn_reactor(&mut self, entity: Entity, callback_id: u64)
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

    /// Queue reactions to a component insertion.
    pub(crate) fn react_to_insertion<C: ReactComponent>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity-specific component reactors
        commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Insertion, entity, TypeId::of::<C>()), react_to_entity_event)
            );

        // entity-agnostic component reactors
        let Some(handlers) = self.component_reactors.get(&TypeId::of::<C>()) else { return; };
        for sys_id in handlers.insertion_callbacks.iter()
        {
            enque_reaction(commands, *sys_id, entity);
        }
    }

    /// Queue reactions to a component mutation.
    pub(crate) fn react_to_mutation<C: ReactComponent>(&mut self, commands: &mut Commands, entity: Entity)
    {
        // entity-specific component reactors
        commands.add(
                move |world: &mut World|
                syscall(world, (EntityReactType::Mutation, entity, TypeId::of::<C>()), react_to_entity_event)
            );

        // entity-agnostic component reactors
        let Some(handlers) = self.component_reactors.get(&TypeId::of::<C>()) else { return; };
        for sys_id in handlers.mutation_callbacks.iter()
        {
            enque_reaction(commands, *sys_id, entity);
        }
    }

    /// React to component removals
    /// - Returns number of callbacks queued.
    /// - Note: We must use a command queue since the react cache is not present in the world, so callbacks may be invalid
    ///   until the react cache is re-inserted. The react cache is removed from the world so we can call removal checkers
    ///   directly (they are type-erased syscalls).
    pub(crate) fn react_to_removals(&mut self, world: &mut World, command_queue: &mut CommandQueue) -> usize
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
                for sys_id in reactors.removal_callbacks.iter()
                {
                    enque_reaction(&mut commands, *sys_id, *entity);
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
    pub(crate) fn react_to_resource_mutation<R: ReactResource>(&mut self, commands: &mut Commands)
    {
        // resource handlers
        let Some(handlers) = self.resource_reactors.get(&TypeId::of::<R>()) else { return; };
        for sys_id in handlers.iter()
        {
            enque_reaction(commands, *sys_id, ());
        }
    }

    /// Queue reactions to an event.
    pub(crate) fn react_to_event<E: 'static>(&mut self, commands: &mut Commands)
    {
        // resource handlers
        let Some(handlers) = self.event_reactors.get(&TypeId::of::<E>()) else { return; };
        for sys_id in handlers.iter()
        {
            enque_reaction(commands, *sys_id, ());
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
            event_reactors        : HashMap::new(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
