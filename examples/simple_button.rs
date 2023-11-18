//local shortcuts
use bevy_kot::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_lunex::prelude::*;

//standard shortcuts


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

fn build_ui(mut ui: UiBuilder<MainUi>)
{
    // root widget
    let root = relative_widget(ui.tree(), "root", (0., 100.), (0., 100.));

    // button widget
    let button = relative_widget(ui.tree(), root.end("button"), (35., 65.), (40., 60.));

    // default button image tied to button
    let default_widget = make_overlay(ui.tree(), &button, "default", true);
    add_button_rect(&mut ui, &default_widget, Color::GRAY);

    // pressed button image tied to button
    let pressed_widget = make_overlay(ui.tree(), &button, "pressed", false);
    add_button_rect(&mut ui, &pressed_widget, Color::DARK_GRAY);

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
        .spawn_with::<MouseLButtonMain>(&mut ui, button.clone())
        .unwrap();
    let mut entity_commands = ui.commands().entity(entity);
    entity_commands.insert(UIInteractionBarrier::<MainUi>::default());

    // button text
    entity_commands.insert(
            TextElementBundle::new(
                    button,
                    TextParams::center()
                        .with_style(&text_style)
                        .with_depth(100.)
                        .with_width(Some(70.)),
                    "HELLO, WORLD!"
                )
        );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    commands.spawn((Cursor::new(0.0), Transform::default(), MainMouseCursor));

    // prepare lunex ui tree
    commands.insert_resource(StyleStackRes::<MainUi>::default());
    commands.spawn((UiTree::new("ui"), MainUi));
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
        .add_plugins(LunexUiPlugin)
        //.add_plugins(UIDebugOverlayPlugin)
        .add_plugins(ReactPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .register_interaction_source(MouseLButtonMain::default())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
