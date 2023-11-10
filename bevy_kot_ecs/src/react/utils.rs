//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::Command;
use bevy::prelude::*;

//standard shortcuts
use core::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper type for callbacks.
///
/// Used to domain-separate named react systems from other named_syscall regimes.
pub(crate) struct ReactCallback<S>(PhantomData<S>);

//-------------------------------------------------------------------------------------------------------------------

/// Queue a command with a call to react to all removals and despawns.
/// - We want to apply any side effects or chained reactions before any sibling reactions/commands.
///
/// Note that we assume the specified command internally handles its deferred state. We don't want to call
/// `apply_deferred` here since the global `apply_deferred` is inefficient.
pub(crate) fn enque_command(commands: &mut Commands, cb: impl Command)
{
    commands.add(
            move |world: &mut World|
            {
                cb.apply(world);
                react_to_all_removals_and_despawns(world);
            }
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Queue a named system with a call to react to all removals and despawns.
/// - We want to apply any side effects or chained reactions before any sibling reactions/commands.
pub(crate) fn enque_reaction<I: Send + Sync + 'static>(commands: &mut Commands, sys_id: SysId, input: I)
{
    commands.add(
            move |world: &mut World|
            {
                let Ok(()) = named_syscall_direct::<I, ()>(world, sys_id, input)
                else { tracing::error!(?sys_id, "recursive reactions are not supported"); return; };
                react_to_all_removals_and_despawns(world);
            }
        );
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn revoke_named_system<I: Send + Sync + 'static>(sys_id: SysId) -> impl FnOnce(&mut World) + Send + Sync + 'static
{
    move |world: &mut World|
    {
        let Some(mut cache) = world.get_resource_mut::<IdMappedSystems<I, ()>>() else { return; };
        cache.revoke_sysid(sys_id);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) enum EntityReactType
{
    Insertion,
    Mutation,
    Removal,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
pub(crate) struct EntityReactors
{
    pub(crate) insertion_callbacks : HashMap<TypeId, Vec<SysId>>,
    pub(crate) mutation_callbacks  : HashMap<TypeId, Vec<SysId>>,
    pub(crate) removal_callbacks   : HashMap<TypeId, Vec<SysId>>,
}

impl EntityReactors
{
    pub(crate) fn is_empty(&self) -> bool
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum ReactorType
{
    EntityInsertion(Entity, TypeId),
    EntityMutation(Entity, TypeId),
    EntityRemoval(Entity, TypeId),
    ComponentInsertion(TypeId),
    ComponentMutation(TypeId),
    ComponentRemoval(TypeId),
    Despawn(Entity),
    ResourceMutation(TypeId),
    Event(TypeId),
}

/// Token for revoking reactors (event reactors use [`EventRevokeToken`]).
///
/// See [`ReactCommands::revoke()`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RevokeToken
{
    pub(crate) reactor_type : ReactorType,
    pub(crate) sys_id       : SysId,
}

//-------------------------------------------------------------------------------------------------------------------
