//local shortcuts

//third-party shortcuts
use bevy::ecs::system::{SystemParam, SystemState, BoxedSystem};
use bevy::prelude::*;
use bevy::utils::{AHasher, HashMap};

use fxhash::FxHasher32;

//standard shortcuts
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// A system caller may have multiple instances. We need to ensure the local state of these instances is
/// not shared. This hashmap allows us to dynamically store instance states.
#[derive(Default, Resource)]
struct StateInstances<T: 'static + SystemParam>
{
    instances: HashMap<CallId, SystemState<T>>,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn init_callable_system<S: 'static + SystemParam>(world: &mut World, id: CallId)
{
    // 1. obtain the callable system's existing state instances or make a new one
    let mut state_instances =
        match world.remove_resource::<StateInstances<S>>()
        {
            Some(s) => s,
            None    =>
            {
                // Note, this message should only appear once! If you see it twice in the logs, the function
                // may have been called recursively, and will panic.
                debug!("Init system state {}", std::any::type_name::<S>());
                StateInstances::<S>{instances: HashMap::new()}
            }
        };

    // 2. make sure our callable system has an instance for this call id
    if !state_instances.instances.contains_key(&id)
    {
        debug!("Registering system state for system caller {id:?} of type {}", std::any::type_name::<S>());
        state_instances.instances.insert(id, SystemState::new(world));
    }

    // 3. add the state instances to the world
    world.insert_resource(state_instances);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// An identifier for [CallableSystem]s. Each identifier represents a unique system context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallId(pub u64);

impl CallId
{
    /// make a new caller id
    pub fn new(name: &str) -> Self
    {
        let bytes      = name.as_bytes();
        let mut hasher = FxHasher32::default();
        hasher.write(bytes);
        CallId(hasher.finish())
    }

    /// make a caller id by extending an existing caller id
    pub fn with(&self, name: &str) -> CallId
    {
        Self::new(&format!("{}{name}", self.0))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Implemented types can be called like normal functions.
/// - accepts one function argument
pub trait CallableSystem: SystemParam
{
    /// Parameter type that allows custom data to be passed from caller to the callable system.
    type ArgT;

    /// Implementation of the callable system.
    fn system(world: &mut World, state: &mut SystemState<Self>, id: CallId, arg: Self::ArgT);
}

/// Implemented types can be called like normal functions.
/// - accepts no function arguments
pub trait BasicCallableSystem: SystemParam
{
    /// Implementation of the callable system for callables with no arguments.
    fn system(world: &mut World, state: &mut SystemState<Self>, id: CallId);
}

//-------------------------------------------------------------------------------------------------------------------

/// Call a callable system (one function argument).
///
/// # Examples
///
/// ```
/// use bevy_kot::ecs::*;
/// use bevy::ecs::system::{SystemParam, SystemState};
/// use bevy::prelude::*;
/// use std::marker::PhantomData;
/// use std::ops::Deref;
/// 
/// #[derive(SystemParam)]
/// pub struct CallableTest<'w, 's>
/// {
///     counter: Local<'s, usize>,
/// 
///     // we must use phantom data to ignore unused lifetime names ('w in this case)
///     #[system_param(ignore)]
///     _phantom: PhantomData<(&'w (), &'s ())>,
/// }
/// 
/// impl<'w, 's> CallableSystem for CallableTest<'w, 's>
/// {
///     type ArgT = usize;
/// 
///     fn system(world: &mut World, state: &mut SystemState<Self>, _id: CallId, test_counter: usize)
///     {
///         // extract the system context
///         let mut context = state.get_mut(world);
/// 
///         // expect counter matches test_counter
///         assert_eq!(*context.counter.deref(), test_counter);
///         *context.counter = *context.counter + 1;
///     }
/// }
/// 
/// let mut world = World::new();
/// 
/// call::<CallableTest>(&mut world, CallId::new("a"), 0);
/// call::<CallableTest>(&mut world, CallId::new("a"), 1);
///
/// call::<CallableTest>(&mut world, CallId::new("b"), 0);
/// call::<CallableTest>(&mut world, CallId::new("b"), 1);
/// ```
///
pub fn call<S: 'static + CallableSystem>(world: &mut World, id: CallId, arg: S::ArgT)
{
    // 1. make sure the callable system has been cached for this call id
    init_callable_system::<S>(world, id);

    // 2. call our cached system
    world.resource_scope(
            | world, mut states: Mut<StateInstances<S>> |
            {
                let cached_state = states.instances.get_mut(&id).unwrap();
                S::system(world, cached_state, id, arg);
                cached_state.apply(world);
            }
        );
}

/// Call a callable system (no function arguments).
///
/// # Examples
///
/// ```
/// use bevy_kot::ecs::*;
/// use bevy::ecs::system::{SystemParam, SystemState};
/// use bevy::prelude::*;
/// use std::marker::PhantomData;
/// use std::ops::Deref;
/// 
/// #[derive(Resource)]
/// struct CallCounter(u16);
/// 
/// #[derive(SystemParam)]
/// pub struct BasicCallableTest<'w, 's>
/// {
///     counter: ResMut<'w, CallCounter>,
/// 
///     // we must use phantom data to ignore unused lifetime names ('w in this case)
///     #[system_param(ignore)]
///     _phantom: PhantomData<(&'w (), &'s ())>,
/// }
/// 
/// impl<'w, 's> BasicCallableSystem for BasicCallableTest<'w, 's>
/// {
///     fn system(world: &mut World, state: &mut SystemState<Self>, _id: CallId)
///     {
///         // extract the system context
///         let mut context = state.get_mut(world);
/// 
///         // increment global counter
///         context.counter.0 = context.counter.0 + 1;
///     }
/// }
///
/// let mut world = World::new();
/// world.insert_resource::<CallCounter>(CallCounter(0));
/// 
/// call_basic::<BasicCallableTest>(&mut world, CallId::new("a"));
/// call_basic::<BasicCallableTest>(&mut world, CallId::new("a"));
///
/// call_basic::<BasicCallableTest>(&mut world, CallId::new("b"));
/// call_basic::<BasicCallableTest>(&mut world, CallId::new("b"));
///
/// let counter = world.remove_resource::<CallCounter>().unwrap();
/// assert_eq!(counter.0, 4);
/// ```
///
pub fn call_basic<S: 'static + BasicCallableSystem>(world: &mut World, id: CallId)
{
    // 1. make sure the callable system has been cached for this call id
    init_callable_system::<S>(world, id);

    // 2. call our cached system
    world.resource_scope(
            | world, mut states: Mut<StateInstances<S>> |
            {
                let cached_state = states.instances.get_mut(&id).unwrap();
                S::system(world, cached_state, id);
                cached_state.apply(world);
            }
        );
}

//-------------------------------------------------------------------------------------------------------------------

/// Execute a system on some data then apply the system's deferred commands.
///
/// # WARNING
/// If a system is called recursively, the Local system parameters of all but the outer-most invocation will not
/// persist.
///
/// # Examples
///
/// ```
/// use bevy_kot::ecs::*;
/// use bevy::prelude::*;
/// 
/// // normal system: takes an input and sets a local
/// fn test_system(In(input): In<u16>, mut local: Local<u16>)
/// {
///     assert_eq!(input, *local);
///     *local += 1;
/// }
/// 
/// let mut world = World::new();
/// 
/// syscall(&mut world, 0u16, test_system);
/// syscall(&mut world, 1u16, test_system);  //Local is preserved
///
/// // function-like system: takes an input and returns an output
/// fn test_function(In(input): In<u16>) -> u16
/// {
///     input * 2
/// }
/// 
/// let mut world = World::new();
/// 
/// assert_eq!(syscall(&mut world, 1u16, test_function), 2u16);  //systems may have outputs as well
/// ```
///
pub fn syscall<I, O, S, Marker>(world: &mut World, input: I, system: S) -> O
where
    I: Send + 'static,
    O: Send + 'static,
    S: IntoSystem<I, O, Marker> + Send + 'static + Sync,
{
    // get the initialized system
    let mut system =
        match world.remove_resource::<InitializedSystem<I, O, S>>()
        {
            Some(system) => system,
            None =>
            {
                let mut sys = IntoSystem::into_system(system);
                sys.initialize(world);
                InitializedSystem::<I, O, S>{ sys: Box::new(sys), _phantom: PhantomData::<S>{} }
            }
        };

    // run the system
    let result = system.sys.run(input, world);

    // apply any pending changes
    system.sys.apply_deferred(world);

    // put the system back
    world.insert_resource(system);

    return result;
}

#[derive(Resource)]
struct InitializedSystem<I, O, S>
where
    I: Send + 'static,
    O: Send + 'static,
    S: Send + 'static + Sync
{
    sys      : BoxedSystem<I, O>,
    _phantom : PhantomData<S>
}

//-------------------------------------------------------------------------------------------------------------------

/// Execute a named system on some data then apply the system's deferred commands.
///
/// Systems with different names will have different Local state.
///
/// # WARNING
/// If a system is called recursively, the Local system parameters of all but the outer-most invocation will not
/// persist.
///
/// # Examples
///
/// ```
/// use bevy_kot::ecs::*;
/// use bevy::prelude::*;
/// 
/// fn test_system(In(input): In<u16>, mut local: Local<u16>) -> u16
/// {
///     *local += input;
///     *local
/// }
/// 
/// let mut world = World::new();
/// 
/// assert_eq!(named_syscall(&mut world, "a", 1u16, test_system), 1);
/// assert_eq!(named_syscall(&mut world, "a", 1u16, test_system), 2);    //Local is preserved
/// assert_eq!(named_syscall(&mut world, "b", 10u16, test_system), 10);  //new Local
/// assert_eq!(named_syscall(&mut world, "b", 10u16, test_system), 20);
/// ```
///
pub fn named_syscall<H, I, O, S, Marker>(
    world  : &mut World,
    id     : H,
    input  : I,
    system : S
) -> O
where
    H: Hash,
    I: Send + 'static,
    O: Send + 'static,
    S: IntoSystem<I, O, Marker> + Send + 'static + Sync,
{
    // the system id
    let sys_id = SysId::new(id);

    // get resource storing the id-mapped systems
    let mut id_mapped_systems = world.get_resource_or_insert_with::<IdMappedSystems<I, O, S>>(
            || IdMappedSystems::default()
        );

    // take the initialized system
    let mut system =
        match id_mapped_systems.systems.get_mut(&sys_id).map_or(None, |node| node.take())
        {
            Some(system) => system,
            None =>
            {
                let mut sys = IntoSystem::into_system(system);
                sys.initialize(world);
                Box::new(sys)
            }
        };

    // run the system
    let result = system.run(input, world);

    // apply any pending changes
    system.apply_deferred(world);

    // re-acquire mutable access to id-mapped systems
    let mut id_mapped_systems = world.get_resource_or_insert_with::<IdMappedSystems<I, O, S>>(
            || IdMappedSystems::default()
        );

    // put the system back
    // - we ignore overwrites
    match id_mapped_systems.systems.get_mut(&sys_id)
    {
        Some(node) => { let _ = node.replace(system); },
        None       => { let _ = id_mapped_systems.systems.insert(sys_id, Some(system)); },
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SysId(u64);

impl SysId
{
    fn new(id: impl Hash) -> Self
    {
        let mut hasher = AHasher::default();
        id.hash(&mut hasher);
        SysId(hasher.finish())
    }
}

#[derive(Resource)]
struct IdMappedSystems<I, O, S>
where
    I: Send + 'static,
    O: Send + 'static,
    S: Send + 'static + Sync
{
    systems  : HashMap<SysId, Option<BoxedSystem<I, O>>>,
    _phantom : PhantomData<S>
}

impl<I, O, S> Default for IdMappedSystems<I, O, S>
where
    I: Send + 'static,
    O: Send + 'static,
    S: Send + 'static + Sync
{
    fn default() -> Self { Self{ systems: HashMap::default(), _phantom: PhantomData::default() } }
}

//-------------------------------------------------------------------------------------------------------------------
