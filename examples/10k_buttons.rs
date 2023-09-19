//path aliases
use bevy_kot::ui as kot;
use bevy_kot::ui::builtin as kot_builtin;
use bevy_lunex::prelude as lunex;

//local shortcuts
use kot::RegisterInteractionSourceExt;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn spawn_button(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    root         : &lunex::Widget,
    x            : f32,
    y            : f32
){
    // button widget
    let button = lunex::Widget::create(
            ui,
            root.end(""),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x, y },
                relative_2 : Vec2 { x: x + 1., y: y + 1. },
                ..Default::default()
            }
        ).unwrap();

    // default button image tied to button
    let default_widget = kot::make_overlay(ui, &button, "", true);
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
    let pressed_widget = kot::make_overlay(ui, &button, "", false);
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
        .press_on_click_or_hold()
        .unpress_on_press_away_recommended()
        .abort_press_on_press_away_if_not_present()
        .build::<kot_builtin::MouseLButtonMain>(&mut entity_commands, button.clone())
        .unwrap();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands, asset_server: Res<AssetServer>)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        );

    // make lunex cursor
    commands.spawn((lunex::Cursor::new(0.0), Transform::default(), kot_builtin::MainMouseCursor));

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

    // spawn 10k buttons
    for x in 0..100
    {
        for y in 0..100
        {
            spawn_button(&mut commands, &asset_server, &mut ui, &root, x as f32, y as f32);
        }
    }

    // add ui tree to ecs (warning: if you queue any UI-dependent callbacks before this, they will fail)
    commands.spawn((ui, kot_builtin::MainUI, kot::UIInteractionBarrier::<kot_builtin::MainUI>::default()));
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
        //.add_plugins(kot::UIDebugOverlayPlugin)
        .register_interaction_source(kot_builtin::MouseLButtonMain::default())
        .add_systems(Startup, setup)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
