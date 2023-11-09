//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::component::Tick;
use bevy::ecs::system::SystemParam;

//standard shortcuts
use core::ops::Deref;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Resource wrapper that enables reacting to resource mutations.
#[derive(Resource)]
struct ReactResInner<R: ReactResource>
{
    resource: R,
}

impl<R: ReactResource> ReactResInner<R>
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
            cache.react_to_resource_mutation::<R>(&mut rcommands.commands);
        }
        &mut self.resource
    }

    /// Mutably access the resource without triggering reactions.
    pub fn get_mut_noreact(&mut self) -> &mut R
    {
        &mut self.resource
    }

    /// Unwrap the resource.
    pub fn take(self) -> R
    {
        self.resource
    }
}

impl<R: ReactResource> Deref for ReactResInner<R>
{
    type Target = R;

    fn deref(&self) -> &R
    {
        &self.resource
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tag trait for reactive resources.
///
/// It is not recommended to add `ReactResource` and `Resource` to the same struct, as it will likely cause confusion.
pub trait ReactResource: Send + Sync + 'static {}

//-------------------------------------------------------------------------------------------------------------------

/// Immutable reader for reactive resources.
#[derive(SystemParam)]
pub struct ReactRes<'w, R: ReactResource>
{
    inner: Res<'w, ReactResInner<R>>,
}

impl<'w, R: ReactResource> DetectChanges for ReactRes<'w, R>
{
    #[inline] fn is_added(&self) -> bool { self.inner.is_added() }
    #[inline] fn is_changed(&self) -> bool { self.inner.is_changed() }
    #[inline] fn last_changed(&self) -> Tick { self.inner.last_changed() }
}

impl<'w, R: ReactResource> Deref for ReactRes<'w, R>
{
    type Target = R;

    fn deref(&self) -> &R
    {
        &self.inner
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mutable wrapper for reactive resources.
#[derive(SystemParam)]
pub struct ReactResMut<'w, R: ReactResource>
{
    inner: ResMut<'w, ReactResInner<R>>,
}

impl<'w, R: ReactResource> ReactResMut<'w, R>
{
    /// Mutably access the resource and trigger reactions.
    pub fn get_mut<'a>(&'a mut self, rcommands: &mut ReactCommands) -> &'a mut R
    {
        self.inner.get_mut(rcommands)
    }

    /// Mutably access the resource without triggering reactions.
    pub fn get_mut_noreact(&mut self) -> &mut R
    {
        self.inner.get_mut_noreact()
    }
}

impl<'w, R: ReactResource> DetectChanges for ReactResMut<'w, R>
{
    #[inline] fn is_added(&self) -> bool { self.inner.is_added() }
    #[inline] fn is_changed(&self) -> bool { self.inner.is_changed() }
    #[inline] fn last_changed(&self) -> Tick { self.inner.last_changed() }
}

impl<'w, R: ReactResource> Deref for ReactResMut<'w, R>
{
    type Target = R;

    fn deref(&self) -> &R
    {
        &self.inner
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ReactResWorldExt
{
    //fn init_react_resource<R: FromWorld>(&mut self) -> ComponentId;
    fn insert_react_resource<R: ReactResource>(&mut self, value: R);

    /*
    fn remove_react_resource<R: ReactResource>(&mut self) -> Option<R>;
    fn contains_react_resource<R: ReactResource>(&self) -> bool;
    fn is_react_resource_added<R: ReactResource>(&self) -> bool;
    fn is_react_resource_changed<R: ReactResource>(&self) -> bool;
    fn react_res<R: ReactResource>(&self) -> &R;
    fn react_res_mut<R: ReactResource>(&mut self) -> &mut R;
    fn react_res_mut_noreact<R: ReactResource>(&mut self) -> &mut R;
    fn get_react_resource<R: ReactResource>(&self) -> Option<&R>;
    fn get_react_resource_mut<R: ReactResource>(&mut self) -> Option<&mut R>;
    fn get_react_resource_mut_noreact<R: ReactResource>(&mut self) -> Option<&mut R>;
    fn get_react_resource_or_insert_with<R: ReactResource>(
        &mut self,
        func: impl FnOnce() -> R,
    ) -> &R;
    */
}

impl ReactResWorldExt for World
{
    //fn init_react_resource<R: FromWorld>(&mut self) -> ComponentId;
    fn insert_react_resource<R: ReactResource>(&mut self, value: R)
    {
        self.insert_resource(ReactResInner::new(value));
    }

    /*
    fn remove_react_resource<R: ReactResource>(&mut self) -> Option<R>;
    fn contains_react_resource<R: ReactResource>(&self) -> bool;
    fn is_react_resource_added<R: ReactResource>(&self) -> bool;
    fn is_react_resource_changed<R: ReactResource>(&self) -> bool;
    fn react_res<R: ReactResource>(&self) -> &R;
    fn react_res_mut<R: ReactResource>(&mut self) -> &mut R;
    fn react_res_mut_noreact<R: ReactResource>(&mut self) -> &mut R;
    fn get_react_resource<R: ReactResource>(&self) -> Option<&R>;
    fn get_react_resource_mut<R: ReactResource>(&mut self) -> Option<&mut R>;
    fn get_react_resource_mut_noreact<R: ReactResource>(&mut self) -> Option<&mut R>;
    fn get_react_resource_or_insert_with<R: ReactResource>(
        &mut self,
        func: impl FnOnce() -> R,
    ) -> &R;
    */
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ReactResAppExt
{
    fn insert_react_resource<R: ReactResource>(&mut self, value: R) -> &mut Self;
}

impl ReactResAppExt for App
{
    fn insert_react_resource<R: ReactResource>(&mut self, value: R) -> &mut Self
    {
        self.world.insert_react_resource(value);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
