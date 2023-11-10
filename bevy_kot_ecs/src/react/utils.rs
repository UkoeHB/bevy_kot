//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::Command;
use bevy::prelude::*;

//standard shortcuts
use core::any::TypeId;
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------

/// Queue a command, followed by a call to `apply_deferred`, then react to all removals and despawns.
/// - We want to apply any side effects or chained reactions before any sibling reactions/commands.
pub(crate) fn enque_command(commands: &mut Commands, cb: impl Command)
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

/// Queue a react callback, followed by a call to `apply_deferred`, then react to all removals and despawns.
/// - We want to apply any side effects or chained reactions before any sibling reactions/commands.
pub(crate) fn enque_reaction<I: Send + Sync + 'static>(commands: &mut Commands, sys_id: SysId, input: I)
{
    commands.add(
            move |world: &mut World|
            {
                let Ok(()) = direct_named_syscall::<I, ()>(world, sys_id, input)
                else { tracing::error!(?sys_id, "recursive reactions are not supported"); return; };
                syscall(world, (), apply_deferred);
                react_to_all_removals_and_despawns(world);
            }
        );
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
