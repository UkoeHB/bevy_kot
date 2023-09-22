//local shortcuts
use bevy_kot::ecs::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct TestComponent(usize);

#[derive(Resource, Default)]
struct TestReactRes(usize);

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct TestReactRecorder(usize);

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Copy test component to recorder
fn update_test_recorder_with_component(
    In(entity)    : In<Entity>,
    mut recorder  : ResMut<TestReactRecorder>,
    test_entities : Query<&React<TestComponent>>,
){
    recorder.0 = test_entities.get(entity).unwrap().0;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn infinitize_test_recorder(mut recorder: ResMut<TestReactRecorder>)
{
    recorder.0 = usize::MAX;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Copy test component to recorder
fn update_test_recorder_with_resource(
    mut recorder  : ResMut<TestReactRecorder>,
    resource      : Res<ReactRes<TestReactRes>>,
){
    recorder.0 = resource.0;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_entity_insertion_reactor(In(entity): In<Entity>, mut react_commands: ReactCommands)
{
    react_commands.add_entity_insertion_reactor::<TestComponent>(
            entity,
            move |world| { syscall(world, entity, update_test_recorder_with_component); }
        );
}

fn add_entity_mutation_reactor(In(entity): In<Entity>, mut react_commands: ReactCommands)
{
    react_commands.add_entity_mutation_reactor::<TestComponent>(
            entity,
            move |world| { syscall(world, entity, update_test_recorder_with_component); }
        );
}

fn add_entity_removal_reactor(In(entity): In<Entity>, mut react_commands: ReactCommands)
{
    react_commands.add_entity_removal_reactor::<TestComponent>(
            entity,
            move |world| { syscall(world, (), infinitize_test_recorder); }
        );
}

fn add_insertion_reactor(mut react_commands: ReactCommands)
{
    react_commands.add_insertion_reactor::<TestComponent>(
            move |world, entity| { syscall(world, entity, update_test_recorder_with_component); }
        );
}

fn add_mutation_reactor(mut react_commands: ReactCommands)
{
    react_commands.add_mutation_reactor::<TestComponent>(
            move |world, entity| { syscall(world, entity, update_test_recorder_with_component); }
        );
}

fn add_removal_reactor(mut react_commands: ReactCommands)
{
    react_commands.add_removal_reactor::<TestComponent>(
            move |world, _entity| { syscall(world, (), infinitize_test_recorder); }
        );
}

fn add_despawn_reactor(In(entity): In<Entity>, mut react_commands: ReactCommands)
{
    react_commands.add_despawn_reactor(
            entity,
            move |world| { syscall(world, (), infinitize_test_recorder); }
        );
}

fn add_resource_mutation_reactor(mut react_commands: ReactCommands)
{
    react_commands.add_resource_mutation_reactor::<TestReactRes>(
            move |world| { syscall(world, (), update_test_recorder_with_resource); }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn insert_on_test_entity(In((entity, component)): In<(Entity, TestComponent)>, mut react_commands: ReactCommands)
{
    react_commands.insert(entity, component);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn remove_from_test_entity(In(entity): In<Entity>, mut commands: Commands)
{
    commands.get_entity(entity).unwrap().remove::<React<TestComponent>>();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_test_entity(
    In((entity, new_val)) : In<(Entity, TestComponent)>,
    mut react_commands    : ReactCommands,
    mut test_entities     : Query<&mut React<TestComponent>>,
){
    *test_entities
        .get_mut(entity)
        .unwrap()
        .get_mut(&mut react_commands) = new_val;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_react_res(
    In(new_val)        : In<usize>,
    mut react_commands : ReactCommands,
    mut react_res      : ResMut<ReactRes<TestReactRes>>
){
    react_res.get_mut(&mut react_commands).0 = new_val;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn pass_component_to_res(
    In(entity)         : In<Entity>,
    mut react_commands : ReactCommands,
    mut react_res      : ResMut<ReactRes<TestReactRes>>,
    test_entities      : Query<&React<TestComponent>>,
){
    react_res.get_mut(&mut react_commands).0 = test_entities.get(entity).unwrap().0;
}

fn add_entity_mutation_reactor_chain_to_res(In(entity): In<Entity>, mut react_commands: ReactCommands)
{
    react_commands.add_entity_mutation_reactor::<TestComponent>(
            entity,
            move |world| { syscall(world, entity, pass_component_to_res); }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_entity_insertion()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, test_entity_a, add_entity_insertion_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);

    // insert (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 2);

    // insert other entity (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(3)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 2);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_component_insertion()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, (), add_insertion_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);

    // insert (reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 2);

    // insert (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(3)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 3);

    // insert (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(4)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 4);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_entity_muation()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, test_entity_a, add_entity_mutation_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(5)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // update (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(10)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 10);

    // update (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);

    // insert other entity (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(100)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);

    // update other entity (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(200)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_component_mutation()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, (), add_mutation_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // update (reaction)
    syscall(&mut world, (test_entity_a, TestComponent(3)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 3);

    // update (reaction)
    syscall(&mut world, (test_entity_b, TestComponent(4)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 4);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_entity_removal()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, test_entity_a, add_entity_removal_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // removal
    syscall(&mut world, test_entity_a, remove_from_test_entity);
    // no immediate reaction
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
    // check for removals (reaction)
    react_to_all_removals_and_despawns(world);
    assert_eq!(world.resource::<TestReactRecorder>().0, usize::MAX);

    // removal of already removed (no reaction)
    *world.resource_mut::<TestReactRecorder>() = TestReactRecorder::default();
    syscall(&mut world, test_entity_a, remove_from_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // removal of other entity (no reaction)
    syscall(&mut world, test_entity_b, remove_from_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_component_removal()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, (), add_removal_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // removal
    syscall(&mut world, test_entity_a, remove_from_test_entity);
    // no immediate reaction
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
    // check for removals (reaction)
    assert_eq!(react_to_removals(world), 1);
    assert_eq!(world.resource::<TestReactRecorder>().0, usize::MAX);

    // removal of already removed (no reaction)
    *world.resource_mut::<TestReactRecorder>() = TestReactRecorder::default();
    syscall(&mut world, test_entity_a, remove_from_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // removal of other entity
    syscall(&mut world, test_entity_b, remove_from_test_entity);
    // no immediate reaction
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
    // check for removals (reaction)
    assert_eq!(react_to_removals(world), 1);
    assert_eq!(world.resource::<TestReactRecorder>().0, usize::MAX);
}

//-------------------------------------------------------------------------------------------------------------------

//react entity despawn
#[test]
fn react_entity_despawn()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactor
    syscall(&mut world, test_entity_a, add_despawn_reactor);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // despawn (reaction)
    assert!(world.despawn(test_entity_a));
    // no immediate reaction
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
    // check for despawns (reaction)
    assert_eq!(add_despawn_reactors(world), 1);
    assert_eq!(world.resource::<TestReactRecorder>().0, usize::MAX);

    // despawn other entity (no reaction)
    *world.resource_mut::<TestReactRecorder>() = TestReactRecorder::default();
    assert!(world.despawn(test_entity_b));
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_resource_mutation()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .insert_resource(ReactRes::new(TestReactRes::default()))
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // add reactor
    syscall(&mut world, (), add_resource_mutation_reactor);
    assert_eq!(world.resource::<ReactRes<TestReactRes>>().0, 0);

    // update resource (reaction)
    syscall(&mut world, 100, update_react_res);
    assert_eq!(world.resource::<ReactRes<TestReactRes>>().0, 100);

    // update resource (reaction)
    syscall(&mut world, 1, update_react_res);
    assert_eq!(world.resource::<ReactRes<TestReactRes>>().0, 1);
}

//-------------------------------------------------------------------------------------------------------------------

//react chain: mutation into mutation
#[test]
fn react_mutation_chain()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .insert_resource(ReactRes::new(TestReactRes::default()))
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity_a = world.spawn_empty().id();
    let test_entity_b = world.spawn_empty().id();

    // add reactors
    syscall(&mut world, test_entity_a, add_entity_mutation_reactor_chain_to_res);
    syscall(&mut world, (), add_resource_mutation_reactor);
    assert_eq!(world.resource::<ReactRes<TestReactRes>>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert other entity (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // update (reaction chain)
    syscall(&mut world, (test_entity_a, TestComponent(3)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 3);

    // update other entity (no reaction reaction)
    syscall(&mut world, (test_entity_b, TestComponent(4)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 3);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
#[should_panic]
fn reactor_panic_without_plugin()
{
    // setup
    let mut app = App::new();
    let mut world = &mut app.world;

    // entity
    let test_entity = world.spawn_empty().id();

    // add reactor (should panic)
    syscall(&mut world, test_entity, add_entity_insertion_reactor);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_pieces_without_plugin()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .insert_resource(ReactRes::new(TestReactRes::default()))
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entity
    let test_entity = world.spawn_empty().id();

    // insert, update, remove
    syscall(&mut world, (test_entity, TestComponent(1)), insert_on_test_entity);
    syscall(&mut world, (test_entity, TestComponent(10)), update_test_entity);
    syscall(&mut world, test_entity, remove_from_test_entity);
    react_to_all_removals_and_despawns(world);

    // despawn
    assert!(world.despawn(test_entity));
    react_to_all_removals_and_despawns(world);

    // update react res
    syscall(&mut world, 100, update_react_res);

    //todo: send event
}

//-------------------------------------------------------------------------------------------------------------------
