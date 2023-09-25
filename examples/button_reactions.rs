//path aliases
use bevy_kot::ecs as kot_ecs;
use bevy_kot::ui as kot;
use bevy_kot::ui::builtin as kot_builtin;
use bevy_lunex::prelude as lunex;

//local shortcuts
use kot::RegisterInteractionSourceExt;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;

//standard shortcuts
use std::fmt::Write;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Default)]
struct ButtonCounter(usize);

impl ButtonCounter
{
    fn increment(&mut self) { self.0 += 1; }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Default)]
struct ReactCounter(usize);

impl ReactCounter
{
    fn increment(&mut self) { self.0 += 1; }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn increment_button_counter(
    mut rcommands : kot_ecs::ReactCommands,
    mut counter   : Query<&mut kot_ecs::React<ButtonCounter>>
){
    counter.get_single_mut()
        .unwrap()
        .get_mut(&mut rcommands)
        .increment();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Increment the react counter whenever the button counter reaches an even number
fn button_counter_reactor(counter: Query<&kot_ecs::React<ButtonCounter>>, mut react_counter: Query<&mut ReactCounter>)
{
    // check if counter is even
    let count: usize = counter.get_single().unwrap().0;
    if (count % 2) != 0 { return; }

    // increment react counter
    react_counter.get_single_mut().unwrap().increment();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_button_counter_text(mut counter: Query<(&mut Text, &kot_ecs::React<ButtonCounter>)>)
{
    let (mut text, counter) = counter.get_single_mut().unwrap();
    text.sections[0].value.clear();
    let _ = write!(text.sections[0].value, "COUNT: {}", counter.0);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_react_counter_text(mut counter: Query<(&mut Text, &ReactCounter)>)
{
    let (mut text, counter) = counter.get_single_mut().unwrap();
    text.sections[0].value.clear();
    let _ = write!(text.sections[0].value, "EVENS: {}", counter.0);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_button(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    button       : lunex::Widget,
){
    // default button image tied to button
    let default_widget = kot::make_overlay(ui, &button, "default", true);
    commands.spawn(
        lunex::ImageElementBundle::new(
                    &default_widget,
                    lunex::ImageParams::center()
                        .with_depth(50.)
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(Color::GRAY),
                    asset_server.load("example_button_rect.png"),
                    Vec2::new(250.0, 142.0)
                )
        );

    // pressed button image tied to button
    let pressed_widget = kot::make_overlay(ui, &button, "pressed", false);
    commands.spawn(
        lunex::ImageElementBundle::new(
                    &pressed_widget,
                    lunex::ImageParams::center()
                        .with_depth(50.)
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(Color::DARK_GRAY),  //tint when pressed
                    asset_server.load("example_button_rect.png"),
                    Vec2::new(250.0, 142.0)
                )
        );

    // button interactivity
    let mut entity_commands = commands.spawn_empty();
    kot::InteractiveElementBuilder::new()
        .with_default_widget(default_widget.clone())
        .with_pressed_widget(pressed_widget)
        .press_on_click()
        .unpress_on_unclick_home_and_abort_on_unclick_away()
        .abort_press_if_obstructed()
        .unpress_callback(|world: &mut World, _: Vec2| kot_ecs::syscall(world, (), increment_button_counter))
        .build::<kot_builtin::MouseLButtonMain>(&mut entity_commands, button.clone())
        .unwrap();
    entity_commands.insert(kot::UIInteractionBarrier::<kot_builtin::MainUI>::default());

    // button text
    let text_style = TextStyle{
            font      : asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size : 40.0,
            color     : Color::WHITE,
        };

    entity_commands.insert(
        lunex::TextElementBundle::new(
                button,
                lunex::TextParams::center()
                    .with_style(&text_style)
                    .with_depth(100.)
                    .with_width(Some(70.)),
                "INCREMENT"
            )
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_count_text(
    rcommands     : &mut kot_ecs::ReactCommands,
    asset_server  : &AssetServer,
    ui            : &mut lunex::UiTree,
    count         : lunex::Widget,
){
    // text widget
    let count_text = lunex::Widget::create(
            ui,
            count.end(""),
            lunex::SolidLayout::new()
            .with_scaling(lunex::SolidScale::Fill),
        ).unwrap();

    let count_text_style = TextStyle {
            font      : asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size : 45.0,
            color     : Color::WHITE,
        };

    let count_entity_commands = rcommands.commands().spawn(
            (
                lunex::TextElementBundle::new(
                    count_text,
                    lunex::TextParams::topleft()
                        .with_style(&count_text_style),
                    "COUNT:  0"  //use initial value to get correct initial text boundary
                ),
            )
        );

    // add reactive button counter
    let entity_id = count_entity_commands.id();
    rcommands.insert(entity_id, ButtonCounter::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_react_count_text(
    rcommands     : &mut kot_ecs::ReactCommands,
    asset_server  : &AssetServer,
    ui            : &mut lunex::UiTree,
    react_count   : lunex::Widget,
){
    // text widget
    let react_count_text = lunex::Widget::create(
            ui,
            react_count.end(""),
            lunex::SolidLayout::new()
            .with_scaling(lunex::SolidScale::Fill),
        ).unwrap();

    let react_count_text_style = TextStyle {
            font      : asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size : 80.0,
            color     : Color::WHITE,
        };

    rcommands.commands().spawn(
            (
                lunex::TextElementBundle::new(
                    react_count_text,
                    lunex::TextParams::topleft()
                        .with_style(&react_count_text_style)
                        .with_width(Some(100.)),
                    "EVENS:  0"  //use initial value to get correct initial text boundary
                ),
                ReactCounter::default(),
            )
        );

    // add reaction
    rcommands.add_mutation_reactor::<ButtonCounter>(
            |world: &mut World, _: Entity| kot_ecs::syscall(world, (), button_counter_reactor)
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut rcommands: kot_ecs::ReactCommands, asset_server: Res<AssetServer>)
{
    // prepare 2D camera
    rcommands.commands().spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    rcommands.commands().spawn((lunex::Cursor::new(0.0), Transform::default(), kot_builtin::MainMouseCursor));

    // create lunex ui tree
    let mut ui = lunex::UiTree::new("ui");

    // root widget
    let root = lunex::Widget::create(
            &mut ui,
            "root",
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 0.0, y: 0.0 },
                relative_2 : Vec2 { x: 100.0, y: 100.0 },
                ..Default::default()
            }
        ).unwrap();

    // add button
    let button = lunex::Widget::create(
            &mut ui,
            root.end("button"),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 40.0, y: 30.0 },
                relative_2 : Vec2 { x: 60.0, y: 45.0 },
                ..Default::default()
            }
        ).unwrap();

    setup_button(rcommands.commands(), &asset_server, &mut ui, button);

    // add count text
    let count = lunex::Widget::create(
            &mut ui,
            root.end("count"),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 45.0, y: 55.0 },
                relative_2 : Vec2 { x: 55.0, y: 65.0 },
                ..Default::default()
            }
        ).unwrap();
    setup_count_text(&mut rcommands, &asset_server, &mut ui, count);

    // add react count text
    let react_count = lunex::Widget::create(
            &mut ui,
            root.end("react_count"),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 45.0, y: 67.0 },
                relative_2 : Vec2 { x: 55.0, y: 77.0 },
                ..Default::default()
            }
        ).unwrap();
    setup_react_count_text(&mut rcommands, &asset_server, &mut ui, react_count);

    // add ui tree to ecs (warning: if you queue any UI-dependent callbacks before this, they will fail)
    rcommands.commands().spawn((ui, kot_builtin::MainUI));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(
            bevy::DefaultPlugins.set(
                WindowPlugin{
                    primary_window: Some(Window{ window_theme: Some(WindowTheme::Dark), ..Default::default() }),
                    ..Default::default()
                }
            )
        )
        .add_plugins(lunex::LunexUiPlugin)
        .add_plugins(kot_ecs::ReactPlugin)
        //.add_plugins(kot::UIDebugOverlayPlugin)
        .register_interaction_source(kot_builtin::MouseLButtonMain::default())
        .add_systems(Startup, setup)
        .add_systems(PostUpdate,
            (
                update_button_counter_text,
                update_react_counter_text,
            )
        )
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
