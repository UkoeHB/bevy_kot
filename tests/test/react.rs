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

fn infinitize_test_recorder(mut recorder: ResMut<TestReactRecorder>)
{
    recorder.0 = usize::MAX;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn test_recorder_div2(mut recorder: ResMut<TestReactRecorder>)
{
    recorder.0 /= 2;
}

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

/// Copy test component to recorder
fn update_test_recorder_with_resource(
    mut recorder : ResMut<TestReactRecorder>,
    resource     : Res<ReactRes<TestReactRes>>,
){
    recorder.0 = resource.0;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_test_recorder_with_event(In(data): In<usize>, mut recorder: ResMut<TestReactRecorder>)
{
    recorder.0 = data;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn on_entity_insertion(In(entity): In<Entity>, mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_entity_insertion::<React<TestComponent>>(
            entity,
            move |world| { syscall(world, entity, update_test_recorder_with_component); }
        )
}

fn on_entity_mutation(In(entity): In<Entity>, mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_entity_mutation::<React<TestComponent>>(
            entity,
            move |world| { syscall(world, entity, update_test_recorder_with_component); }
        )
}

fn on_entity_removal(In(entity): In<Entity>, mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_entity_removal::<React<TestComponent>>(
            entity,
            move |world| { syscall(world, (), infinitize_test_recorder); }
        )
}

fn on_insertion(mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_insertion::<React<TestComponent>>(
            move |world, entity| { syscall(world, entity, update_test_recorder_with_component); }
        )
}

fn on_mutation(mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_mutation::<React<TestComponent>>(
            move |world, entity| { syscall(world, entity, update_test_recorder_with_component); }
        )
}

fn on_removal(mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_removal::<React<TestComponent>>(
            move |world, _entity| { syscall(world, (), infinitize_test_recorder); }
        )
}

fn on_despawn(In(entity): In<Entity>, mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_despawn(
            entity,
            move |world| { syscall(world, (), infinitize_test_recorder); }
        ).unwrap()
}

fn on_despawn_div2(In(entity): In<Entity>, mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_despawn(
            entity,
            move |world| { syscall(world, (), test_recorder_div2); }
        ).unwrap()
}

fn on_resource_mutation(mut rcommands: ReactCommands) -> RevokeToken
{
    rcommands.on_resource_mutation::<ReactRes<TestReactRes>>(
            move |world| { syscall(world, (), update_test_recorder_with_resource); }
        )
}

fn on_event(mut rcommands: ReactCommands) -> EventRevokeToken<usize>
{
    rcommands.on_event::<usize>(
            move |world, event| { syscall(world, *event.get(), update_test_recorder_with_event); }
        )
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn insert_on_test_entity(In((entity, component)): In<(Entity, TestComponent)>, mut rcommands: ReactCommands)
{
    rcommands.insert(entity, component);
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
    mut rcommands         : ReactCommands,
    mut test_entities     : Query<&mut React<TestComponent>>,
){
    *test_entities
        .get_mut(entity)
        .unwrap()
        .get_mut(&mut rcommands) = new_val;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_react_res(
    In(new_val)   : In<usize>,
    mut rcommands : ReactCommands,
    mut react_res : ResMut<ReactRes<TestReactRes>>
){
    react_res.get_mut(&mut rcommands).0 = new_val;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_data_event(In(data): In<usize>, mut rcommands: ReactCommands)
{
    rcommands.send(data);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn pass_component_to_res(
    In(entity)    : In<Entity>,
    mut rcommands : ReactCommands,
    mut react_res : ResMut<ReactRes<TestReactRes>>,
    test_entities : Query<&React<TestComponent>>,
){
    react_res.get_mut(&mut rcommands).0 = test_entities.get(entity).unwrap().0;
}

fn on_entity_mutation_chain_to_res(In(entity): In<Entity>, mut rcommands: ReactCommands)
{
    rcommands.on_entity_mutation::<React<TestComponent>>(
            entity,
            move |world| { syscall(world, entity, pass_component_to_res); }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn revoke_reactor(In(token): In<RevokeToken>, mut rcommands: ReactCommands)
{
    rcommands.revoke(token);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn revoke_event_reactor(In(token): In<EventRevokeToken<usize>>, mut rcommands: ReactCommands)
{
    rcommands.revoke_event_reactor(token);
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
    syscall(&mut world, test_entity_a, on_entity_insertion);
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
    syscall(&mut world, (), on_insertion);
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
    syscall(&mut world, test_entity_a, on_entity_mutation);
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
    syscall(&mut world, (), on_mutation);
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
    syscall(&mut world, test_entity_a, on_entity_removal);
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
    syscall(&mut world, (), on_removal);
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
    syscall(&mut world, test_entity_a, on_despawn);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // check for despawns (no reaction before despawn)
    assert_eq!(react_to_despawns(world), 0);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // despawn (reaction)
    assert!(world.despawn(test_entity_a));
    // no immediate reaction
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
    // check for despawns (reaction)
    assert_eq!(react_to_despawns(world), 1);
    assert_eq!(world.resource::<TestReactRecorder>().0, usize::MAX);

    // despawn other entity (no reaction)
    *world.resource_mut::<TestReactRecorder>() = TestReactRecorder::default();
    assert!(world.despawn(test_entity_b));
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_entity_despawn_multiple_reactors()
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
    syscall(&mut world, test_entity_a, on_despawn);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // add second reactor
    syscall(&mut world, test_entity_a, on_despawn_div2);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_a, TestComponent(1)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity_b, TestComponent(2)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // check for despawns (no reaction before despawn)
    assert_eq!(react_to_despawns(world), 0);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // despawn (reaction)
    assert!(world.despawn(test_entity_a));
    // no immediate reaction
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);
    // check for despawns (reaction)
    assert_eq!(react_to_despawns(world), 2);
    assert_eq!(world.resource::<TestReactRecorder>().0, usize::MAX / 2);

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
    syscall(&mut world, (), on_resource_mutation);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // update resource (reaction)
    syscall(&mut world, 100, update_react_res);
    assert_eq!(world.resource::<TestReactRecorder>().0, 100);

    // update resource (reaction)
    syscall(&mut world, 1, update_react_res);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_data_event()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // add reactor
    syscall(&mut world, (), on_event);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // send event (reaction)
    syscall(&mut world, 222, send_data_event);
    assert_eq!(world.resource::<TestReactRecorder>().0, 222);

    // send event (reaction)
    syscall(&mut world, 1, send_data_event);
    assert_eq!(world.resource::<TestReactRecorder>().0, 1);
}

//-------------------------------------------------------------------------------------------------------------------

//react chain: component mutation into resource mutation
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
    syscall(&mut world, test_entity_a, on_entity_mutation_chain_to_res);
    syscall(&mut world, (), on_resource_mutation);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

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
    syscall(&mut world, test_entity, on_entity_insertion);
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

#[test]
fn revoke_entity_mutation_reactor()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity = world.spawn_empty().id();

    // add reactor
    let token = syscall(&mut world, test_entity, on_entity_mutation);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity, TestComponent(5)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // update (reaction)
    syscall(&mut world, (test_entity, TestComponent(10)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 10);

    // revoke
    syscall(&mut world, token, revoke_reactor);

    // update (no reaction)
    syscall(&mut world, (test_entity, TestComponent(1)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 10);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn revoke_component_mutation_reactor()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // entities
    let test_entity = world.spawn_empty().id();

    // add reactor
    let token = syscall(&mut world, (), on_mutation);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // insert (no reaction)
    syscall(&mut world, (test_entity, TestComponent(5)), insert_on_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // update (reaction)
    syscall(&mut world, (test_entity, TestComponent(10)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 10);

    // revoke
    syscall(&mut world, token, revoke_reactor);

    // update (no reaction)
    syscall(&mut world, (test_entity, TestComponent(1)), update_test_entity);
    assert_eq!(world.resource::<TestReactRecorder>().0, 10);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn revoke_data_event_reactor()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // add reactor
    let revoke_token = syscall(&mut world, (), on_event);
    assert_eq!(world.resource::<TestReactRecorder>().0, 0);

    // send event (reaction)
    syscall(&mut world, 222, send_data_event);
    assert_eq!(world.resource::<TestReactRecorder>().0, 222);

    // revoke reactor
    syscall(&mut world, revoke_token, revoke_event_reactor);

    // send event (no reaction)
    syscall(&mut world, 1, send_data_event);
    assert_eq!(world.resource::<TestReactRecorder>().0, 222);
}

//-------------------------------------------------------------------------------------------------------------------
