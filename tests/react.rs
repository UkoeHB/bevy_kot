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
struct TestReactRecorder(usize);


#[derive(Resource, Default)]
struct TestReactRes(usize);

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_test_recorder(
    In(entity)    : In<Entity>,
    mut recorder  : ResMut<TestReactRecorder>,
    test_entities : Query<&React<TestComponent>>,
){
    recorder.0 = test_entities.get(entity).unwrap().0;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn double_react_res_value(mut react_res: ResMut<ReactRes<TestReactRes>>)
{
    react_res.get_mut_noreact().0 *= 2;  //noreact otherwise it will infinitely loop
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn spawn_test_entity(In(component): In<TestComponent>, mut react_commands: ReactCommands) -> Entity
{
    let entity = react_commands.commands().spawn_empty().id();
    react_commands.insert(entity, component);
    entity
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_reactor(In(entity): In<Entity>, mut react_commands: ReactCommands)
{
    react_commands.react_to_mutation_on_entity::<TestComponent>(
            entity,
            move |world| { syscall(world, entity, update_test_recorder); }
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_test_entity(
    In((entity, component)) : In<(Entity, TestComponent)>,
    mut react_commands      : ReactCommands,
    mut test_entities       : Query<&mut React<TestComponent>>,
){
    *test_entities.get_mut(entity).unwrap().get_mut(&mut react_commands) = component;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_res_reactor(mut react_commands: ReactCommands)
{
    react_commands.react_to_resource_mutation::<TestReactRes>(|world| syscall(world, (), double_react_res_value));
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

#[test]
fn react_basic()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .init_resource::<TestReactRecorder>();
    let mut world = &mut app.world;

    // react cycle
    let test_entity = syscall(&mut world, TestComponent(0), spawn_test_entity);
    syscall(&mut world, test_entity, add_reactor);
    syscall(&mut world, (test_entity, TestComponent(10)), update_test_entity);

    // check
    assert_eq!(world.resource::<TestReactRecorder>().0, 10);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn react_res()
{
    // setup
    let mut app = App::new();
    app.add_plugins(ReactPlugin)
        .insert_resource(ReactRes::new(TestReactRes::default()));
    let mut world = &mut app.world;

    // react cycle
    syscall(&mut world, (), add_res_reactor);
    syscall(&mut world, 100, update_react_res);

    // check
    assert_eq!(world.resource::<ReactRes<TestReactRes>>().0, 200);
}

//-------------------------------------------------------------------------------------------------------------------
