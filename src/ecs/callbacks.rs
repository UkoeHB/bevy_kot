//local shortcuts

//third-party shortcuts
use bevy::ecs::system::Command;
use bevy::prelude::*;

//standard shortcuts
use std::marker::PhantomData;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// Callback wrapper. Implements `Command`.
/// - The type `T` can be used to mark the callback for query filtering.
#[derive(Component)]
pub struct Callback<T: Send + Sync + 'static>
{
    callback : Arc<dyn Fn(&mut World) -> () + Send + Sync + 'static>,
    _phantom : PhantomData<T>,
}

impl<T: Send + Sync + 'static> Clone for Callback<T>
{ fn clone(&self) -> Self { Self{ callback: self.callback.clone(), _phantom: PhantomData::default() } } }

impl<T: Send + Sync + 'static> Callback<T>
{
    pub fn new(callback: impl Fn(&mut World) -> () + Send + Sync + 'static) -> Self
    {
        Self{ callback: Arc::new(callback), _phantom: PhantomData::default() }
    }
}

impl<T: Send + Sync + 'static> Command for Callback<T>
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Callback wrapper that lets you call with a value. The helper returned by `.call_with()` implements `Command`.
/// - The type `T` can be used to mark the callback for query filtering.
#[derive(Component)]
pub struct CallbackWith<T: Send + Sync + 'static, V>
{
    callback : Arc<dyn Fn(&mut World, V) -> () + Send + Sync + 'static>,
    _phantom : PhantomData<T>,
}

impl<T: Send + Sync + 'static, V> Clone for CallbackWith<T, V>
{ fn clone(&self) -> Self { Self{ callback: self.callback.clone(), _phantom: PhantomData::default() } } }

impl<T: Send + Sync + 'static, V: Send + Sync + 'static> CallbackWith<T, V>
{
    pub fn new(callback: impl Fn(&mut World, V) -> () + Send + Sync + 'static) -> Self
    {
        Self{ callback: Arc::new(callback), _phantom: PhantomData::default() }
    }

    pub fn call_with(&self, call_value: V) -> Callwith<T, V>
    {
        Callwith{ callback: self.callback.clone(), call_value, _phantom: PhantomData::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Callback wrapper with a specific call value baked in. Implements `Command`.
/// - The type `T` can be used to mark the callback for query filtering.
pub struct Callwith<T: Send + Sync + 'static, C>
{
    callback   : Arc<dyn Fn(&mut World, C) -> () + Send + Sync + 'static>,
    call_value : C,
    _phantom   : PhantomData<T>,
}

impl<T: Send + Sync + 'static, C> Callwith<T, C>
{
    pub fn new(callback: impl Fn(&mut World, C) -> () + Send + Sync + 'static, call_value: C) -> Self
    {
        Self{ callback: Arc::new(callback), call_value, _phantom: PhantomData::default() }
    }
}

impl<T: Send + Sync + 'static, C: Send + Sync + 'static> Command for Callwith<T, C>
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world, self.call_value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Callback wrapper that mimics `syscall`.
/// - The type `T` can be used to mark the callback for query filtering.
#[derive(Component)]
pub struct SysCall<T: Send + Sync + 'static, I, O>
{
    callback : Arc<dyn Fn(&mut World, I) -> O + Send + Sync + 'static>,
    _phantom : PhantomData<T>,
}

impl<T: Send + Sync + 'static, I, O> Clone for SysCall<T, I, O>
{ fn clone(&self) -> Self { Self{ callback: self.callback.clone(), _phantom: PhantomData::default() } } }

impl<T: Send + Sync + 'static, I, O> SysCall<T, I, O>
{
    pub fn new(callback: impl Fn(&mut World, I) -> O + Send + Sync + 'static) -> Self
    {
        Self{ callback: Arc::new(callback), _phantom: PhantomData::default() }
    }

    pub fn call(&self, world: &mut World, in_val: I) -> O
    {
        (self.callback)(world, in_val)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if the callback was invoked.
pub fn try_callback<C: Send + Sync + 'static>(world: &mut World, entity: Entity) -> bool
{
    let Some(entity_mut) = world.get_entity_mut(entity) else { return false; };
    let Some(cb) = entity_mut.get::<Callback<C>>() else { return false; };
    cb.clone().apply(world);
    true
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if the callback was invoked with the provided value.
pub fn try_callback_with<C, V>(world: &mut World, entity: Entity, value: V) -> bool
where
    C: Send + Sync + 'static,
    V: Send + Sync + 'static
{
    let Some(entity_mut) = world.get_entity_mut(entity) else { return false; };
    let Some(cb) = entity_mut.get::<CallbackWith<C, V>>() else { return false; };
    cb.call_with(value).apply(world);
    true
}

//-------------------------------------------------------------------------------------------------------------------
