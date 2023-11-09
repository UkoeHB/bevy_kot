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

/// Queue a react callback, followed by a call to `apply_deferred`, then react to all removals and despawns.
/// - We want to apply any side effects or chained reactions before any sibling reactions/commands.
pub(crate) fn enque_reaction(commands: &mut Commands, cb: impl Command)
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
    pub(crate) insertion_callbacks : HashMap<TypeId, Vec<(u64, Callback<()>)>>,
    pub(crate) mutation_callbacks  : HashMap<TypeId, Vec<(u64, Callback<()>)>>,
    pub(crate) removal_callbacks   : HashMap<TypeId, Vec<(u64, Callback<()>)>>,
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
}

/// Token for revoking reactors (event reactors use [`EventRevokeToken`]).
///
/// See [`ReactCommands::revoke()`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RevokeToken
{
    pub(crate) reactor_type : ReactorType,
    pub(crate) callback_id  : u64,
}

//-------------------------------------------------------------------------------------------------------------------

/// Token for revoking event reactors (non-event reactors use [`RevokeToken`]).
///
/// See [`ReactCommands::revoke_event_reactor()`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EventRevokeToken<E>
{
    pub(crate) callback_id : u64,
    pub(crate) _p          : PhantomData<E>
}

//-------------------------------------------------------------------------------------------------------------------
