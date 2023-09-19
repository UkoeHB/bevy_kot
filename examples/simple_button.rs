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

    // button widget
    let button = lunex::Widget::create(
            &mut ui,
            root.end("button"),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 35.0, y: 40.0 },
                relative_2 : Vec2 { x: 65.0, y: 60.0 },
                ..Default::default()
            }
        ).unwrap();

    // default button image tied to button
    let default_widget = kot::make_overlay(&mut ui, &button, "default", true);
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
    let pressed_widget = kot::make_overlay(&mut ui, &button, "pressed", false);
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
        .abort_press_on_press_away_if_not_present()
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
                "HELLO, WORLD!"
            )
    );

    // add ui tree to ecs (warning: if you queue any UI-dependent callbacks before this, they will fail)
    commands.spawn((ui, kot_builtin::MainUI));
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
