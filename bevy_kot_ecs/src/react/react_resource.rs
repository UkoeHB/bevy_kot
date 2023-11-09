//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts
use core::ops::Deref;

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
