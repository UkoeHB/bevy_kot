//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::{Command, SystemParam};
use bevy::prelude::*;

//standard shortcuts
use core::any::TypeId;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn revoke_named_system<I: Send + Sync + 'static>(sys_id: SysId) -> impl FnOnce(&mut World) + Send + Sync + 'static
{
    move |world: &mut World|
    {
        let Some(mut cache) = world.get_resource_mut::<IdMappedSystems<I, ()>>() else { return; };
        cache.revoke_sysid(sys_id);
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
        sys_id
    ))                  : In<(Entity, EntityReactType, TypeId, SysId)>,
    mut commands        : Commands,
    mut entity_reactors : Query<&mut EntityReactors>,
    mut cached_systems  : ResMut<IdMappedSystems<(), ()>>,
){
    // revoke cached system
    cached_systems.revoke_sysid(sys_id);

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
    for (idx, id) in callbacks.iter().enumerate()
    {
        if *id != sys_id { continue; }
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

/// Drives reactivity.
///
/// Typically used with [`ReactPlugin`]. If [`ReactPlugin`] is not present:
/// - Adding a reactor will panic.
/// - Reactions will not be triggered.
/// - Sending a [`ReactEvent`] will do nothing.
///
/// Note that each time you register a reactor, it is assigned a unique system state (unique `Local`s). To avoid
/// leaking memory, be sure to revoke reactors when you are done with them. Despawn reactors are automatically cleaned up.
///
/// ## Ordering and determinism
///
/// `ReactCommands` requires exclusive access to an internal cache, which means the order of react events is fully
/// specified. Reactors of the same type will react to an event in the order they are added, and react commands will
/// be applied in the order they were invoked (note that all reactor registration is deferred).
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

    /// Insert a [`ReactComponent`] to the specified entity. It can be queried with [`React<C>`].
    /// - Reactions are enacted after `apply_deferred` is invoked.
    /// - Does nothing if the entity does not exist.
    //todo: consider more ergonomic entity access, e.g. ReactEntityCommands
    pub fn insert<C: ReactComponent>(&mut self, entity: Entity, component: C)
    {
        let Some(mut entity_commands) = self.commands.get_entity(entity) else { return; };
        entity_commands.insert( React{ entity, component } );

        let Some(ref mut cache) = self.cache else { return; };
        cache.react_to_insertion::<C>(&mut self.commands, entity);
    }

    /// Send a react event.
    /// - Reactions are enacted after `apply_deferred` is invoked.
    /// - Any reactors to this event will obtain a [`ReactEvent`] wrapping the event value.
    pub fn send<E: Send + Sync + 'static>(&mut self, event: E)
    {
        let Some(ref mut cache) = self.cache else { return; };
        cache.react_to_event(&mut self.commands, event);
    }

    /// Trigger resource mutation reactions.
    ///
    /// Useful for initializing state after a reactor is registered.
    pub fn trigger_resource_mutation<R: ReactResource + Send + Sync + 'static>(&mut self)
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };
        cache.react_to_resource_mutation::<R>(&mut self.commands);
    }

    /// Revoke a reactor.
    /// - Entity reactors: revoked after `apply_deferred` is invoked.
    /// - Component, despawn, resource, event reactors: revoked immediately.
    pub fn revoke(&mut self, token: RevokeToken)
    {
        let Some(ref mut cache) = self.cache else { panic!("reactors are unsupported without ReactPlugin"); };

        let sys_id = token.sys_id;
        match token.reactor_type
        {
            ReactorType::EntityInsertion(entity, comp_id) =>
            {
                self.commands.add(
                        move |world: &mut World|
                        syscall(world, (entity, EntityReactType::Insertion, comp_id, sys_id), revoke_entity_reactor)
                    );
            }
            ReactorType::EntityMutation(entity, comp_id) =>
            {
                self.commands.add(
                        move |world: &mut World|
                        syscall(world, (entity, EntityReactType::Mutation, comp_id, sys_id), revoke_entity_reactor)
                    );
            }
            ReactorType::EntityRemoval(entity, comp_id) =>
            {
                self.commands.add(
                        move |world: &mut World|
                        syscall(world, (entity, EntityReactType::Removal, comp_id, sys_id), revoke_entity_reactor)
                    );
            }
            ReactorType::ComponentInsertion(comp_id) =>
            {
                cache.revoke_component_reactor(EntityReactType::Insertion, comp_id, sys_id);
                self.commands.add(revoke_named_system::<Entity>(sys_id));
            }
            ReactorType::ComponentMutation(comp_id) =>
            {
                cache.revoke_component_reactor(EntityReactType::Mutation, comp_id, sys_id);
                self.commands.add(revoke_named_system::<Entity>(sys_id));
            }
            ReactorType::ComponentRemoval(comp_id) =>
            {
                cache.revoke_component_reactor(EntityReactType::Removal, comp_id, sys_id);
                self.commands.add(revoke_named_system::<Entity>(sys_id));
            }
            ReactorType::Despawn(entity) =>
            {
                cache.revoke_despawn_reactor(entity, sys_id.id());
                // note: despawn reactors are not registered as named systems
            }
            ReactorType::ResourceMutation(res_id) =>
            {
                cache.revoke_resource_mutation_reactor(res_id, sys_id);
                self.commands.add(revoke_named_system::<()>(sys_id));
            }
            ReactorType::Event(event_id) =>
            {
                let Some(revoker) = cache.revoke_event_reactor(event_id, sys_id) else { return; };
                self.commands.add(move |world: &mut World| revoker.apply(world));
            }
        }
    }

    /// Register a reactor to an ECS change.
    pub fn on<I, R: ReactorRegistrator<Input = I>, Marker>(
        &mut self,
        registrator : R,
        reactor     : impl IntoSystem<I, (), Marker> + Send + Sync + 'static
    ) -> RevokeToken
    {
        registrator.register(self, reactor)
    }
}

//-------------------------------------------------------------------------------------------------------------------
