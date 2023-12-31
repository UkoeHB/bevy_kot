//local shortcuts
use bevy_kot::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowTheme};
use bevy_lunex::prelude::*;

//standard shortcuts
use std::fmt::Write;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Counter for the COUNT text element. Inserted via `ReactCommands` so that mutations will trigger reactions.
#[derive(ReactResource, Default)]
struct ButtonCounter(usize);

impl ButtonCounter
{
    fn increment(&mut self) { self.0 += 1; }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Counter for the EVENS text element. Updated in response to mutations of the `ButtonCounter`.
#[derive(Component, Default)]
struct ReactCounter(usize);

impl ReactCounter
{
    fn increment(&mut self) { self.0 += 1; }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Callback for the button.
fn increment_button_counter(
    mut rcommands : ReactCommands,
    mut counter   : ReactResMut<ButtonCounter>,
){
    counter.get_mut(&mut rcommands).increment();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Reactor for mutations of `React<ButtonCounter>`.
/// - Increment the react counter whenever the button counter reaches an even number
fn button_counter_reactor(counter: ReactRes<ButtonCounter>, mut react_counter: Query<&mut ReactCounter>)
{
    // check if counter is even
    if (counter.0 % 2) != 0 { return; }

    // increment react counter
    react_counter.get_single_mut().unwrap().increment();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Transfer button count into the text element.
fn update_button_counter_text(
    In(text_entity) : In<Entity>,
    counter         : ReactRes<ButtonCounter>,
    mut text        : Query<&mut Text>,
){
    let mut text = text.get_mut(text_entity).unwrap();
    text.sections[0].value.clear();
    let _ = write!(text.sections[0].value, "COUNT: {}", counter.0);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Transfer react count into the text element.
fn update_react_counter_text(mut counter: Query<(&mut Text, &ReactCounter), Changed<ReactCounter>>)
{
    if counter.is_empty() { return; }
    let (mut text, counter) = counter.get_single_mut().unwrap();
    text.sections[0].value.clear();
    let _ = write!(text.sections[0].value, "EVENS: {}", counter.0);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_button_rect(ui: &mut UiBuilder<MainUi>, area: &Widget, color: Color)
{
    let image = ImageElementBundle::new(
            area,
            ImageParams::center()
                .with_width(Some(100.))
                .with_height(Some(100.))
                .with_color(color),
            ui.asset_server.load("example_button_rect.png"),
            Vec2::new(250.0, 142.0)
        );
    ui.commands().spawn(image);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_button(ui: &mut UiBuilder<MainUi>, button: Widget)
{
    // default button image tied to button
    let default_widget = make_overlay(ui.tree(), &button, "default", true);
    add_button_rect(ui, &default_widget, Color::GRAY);

    // pressed button image tied to button
    let pressed_widget = make_overlay(ui.tree(), &button, "pressed", false);
    add_button_rect(ui, &pressed_widget, Color::DARK_GRAY);

    // get text style
    let text_style = TextStyle{
        font      : ui.asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size : 40.0,
        color     : Color::WHITE,
    };

    // button interactivity
    let entity = InteractiveElementBuilder::new()
        .with_default_widget(default_widget)
        .with_pressed_widget(pressed_widget)
        .press_on_click()
        .unpress_on_unclick_home_and_abort_on_unclick_away()
        .abort_press_if_obstructed()
        .on_unpress(increment_button_counter)
        .spawn_with::<MouseLButtonMain>(ui,  button.clone())
        .unwrap();
    let mut entity_commands = ui.commands().entity(entity);
    entity_commands.insert(UiInteractionBarrier::<MainUi>::default());

    // button text
    entity_commands.insert(
        TextElementBundle::new(
                button,
                TextParams::center()
                    .with_style(&text_style)
                    .with_depth(100.)
                    .with_width(Some(70.)),
                "INCREMENT"
            )
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_count_text(ui: &mut UiBuilder<MainUi>, count: Widget)
{
    // text widget
    let count_text = Widget::create(
            ui.tree(),
            count.end("count"),
            SolidLayout::new()
                .with_scaling(SolidScale::Fill),
        ).unwrap();

    let count_text_style = TextStyle {
            font      : ui.asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size : 45.0,
            color     : Color::WHITE,
        };

    let count_entity = ui.commands().spawn(
            (
                TextElementBundle::new(
                    count_text,
                    TextParams::topleft()
                        .with_style(&count_text_style),
                    "COUNT:  0"  //use initial value to get correct initial text boundary
                ),
            )
        ).id();

    // add reactive counter
    ui.commands().insert_react_resource(ButtonCounter::default());

    // update button counter text on mutation
    ui.rcommands.on(resource_mutation::<ButtonCounter>(),
            move |world: &mut World| syscall(world, count_entity, update_button_counter_text)
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_react_count_text(ui: &mut UiBuilder<MainUi>, react_count: Widget)
{
    // text widget
    let react_count_text = Widget::create(
            ui.tree(),
            react_count.end("count_text"),
            SolidLayout::new()
            .with_scaling(SolidScale::Fill),
        ).unwrap();

    let react_count_text_style = TextStyle {
            font      : ui.asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size : 80.0,
            color     : Color::WHITE,
        };

    ui.rcommands.commands().spawn(
            (
                TextElementBundle::new(
                    react_count_text,
                    TextParams::topleft()
                        .with_style(&react_count_text_style),
                    "EVENS:  0"  //use initial value to get correct initial text boundary
                ),
                ReactCounter::default(),
            )
        );

    // add reactor
    ui.rcommands.on(resource_mutation::<ButtonCounter>(),
            |world: &mut World| syscall(world, (), button_counter_reactor)
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut ui: UiBuilder<MainUi>)
{
    // root widget
    let root = relative_widget(ui.tree(), "root", (0., 100.), (0., 100.));

    // add button
    let button = relative_widget(ui.tree(), root.end("button"), (40., 60.), (30., 45.));
    setup_button(&mut ui, button);

    // add count text
    let count = relative_widget(ui.tree(), root.end("count"), (45., 55.), (55., 65.));
    setup_count_text(&mut ui, count);

    // add react count text
    let react_count = relative_widget(ui.tree(), root.end("react_count"), (45., 55.), (67., 77.));
    setup_react_count_text(&mut ui, react_count);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands, window: Query<Entity, (With<Window>, With<PrimaryWindow>)>)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    commands.spawn((Cursor::new(), Transform::default(), Visibility::default(), MainMouseCursor));

    // prepare lunex ui tree
    commands.insert_resource(StyleStackRes::<MainUi>::default());
    let tree = UiTree::<MainUi>::new("ui");

    let window = window.single();
    commands.entity(window).insert(tree.bundle());
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
        .add_plugins(LunexUiPlugin2D::<MainUi>::new())
        //.add_plugins(UIDebugOverlayPlugin)
        .add_plugins(ReactPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .register_interaction_source(MouseLButtonMain::default())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .add_systems(PostUpdate, update_react_counter_text)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
