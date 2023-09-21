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
    react_commands.react_to_mutation::<TestComponent>(
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
