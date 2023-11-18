//local shortcuts
use bevy_kot::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_lunex::prelude::*;

//standard shortcuts
use std::fmt::Write;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct FPSIndicator;

/// Realtime systems
fn refresh_fps_indicator(
    mut indicator_query : Query<&mut Text, With<FPSIndicator>>,
    fps_tracker         : Res<FPSTracker>
){
    // 1. only refresh once per second
    if fps_tracker.current_time().as_secs() <= fps_tracker.previous_time().as_secs() { return }

    // 2. refresh
    let indicator_value = &mut indicator_query.single_mut().sections[0].value;
    indicator_value.clear();
    let _ = write!(indicator_value, "FPS: {}", fps_tracker.fps());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_fps_section(ui: &mut UiBuilder<MainUi>, area: Widget)
{
    // fps layout helper
    let layout_helper = Widget::create(
            ui.tree(),
            area.end(""),
            RelativeLayout{  //add slight buffer around edge; extend y-axis to avoid resizing issues
                absolute_1: Vec2 { x: 5., y: 5. },
                absolute_2: Vec2 { x: -5., y: 0. },
                relative_1: Vec2 { x: 0., y: 0. },
                relative_2: Vec2 { x: 100., y: 200. },
                ..Default::default()
            }
        ).unwrap();

    // fps text widget
    let fps_text = Widget::create(
            ui.tree(),
            layout_helper.end(""),
            SolidLayout::new()
                .with_horizontal_anchor(1.0)
                .with_vertical_anchor(-1.0),
        ).unwrap();

    let fps_text_style = TextStyle {
            font      : ui.asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size : 45.0,
            color     : Color::WHITE,
        };

    ui.commands().spawn(
            (
                TextElementBundle::new(
                    fps_text,
                    TextParams::topleft()
                        .with_style(&fps_text_style)
                        .with_depth(100.0),  //add depth so fps text is higher than buttons
                    "FPS: 999"  //use initial value to get correct initial text boundary
                ),
                FPSIndicator
            )
        );
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

fn spawn_button(ui: &mut UiBuilder<MainUi>, area: &Widget, x: f32, y: f32)
{
    // button widget
    let button = relative_widget(ui.tree(), area.end(""), (x, x + 1.), (y, y + 1.));

    // default button image tied to button
    let default_widget = make_overlay(ui.tree(), &button, "", true);
    add_button_rect(ui, &default_widget, Color::GRAY);

    // pressed button image tied to button
    let pressed_widget = make_overlay(ui.tree(), &button, "", false);
    add_button_rect(ui, &pressed_widget, Color::DARK_GRAY);

    // button interactivity
    InteractiveElementBuilder::new()
        .with_default_widget(default_widget)
        .with_pressed_widget(pressed_widget)
        .press_on_click_or_hold()
        .unpress_on_press_away_or_unclick_any()
        .abort_press_if_obstructed()
        .spawn_with::<MouseLButtonMain>(ui, button.clone())
        .unwrap();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut ui: UiBuilder<MainUi>)
{
    // root widget
    let root = relative_widget(ui.tree(), "root", (0., 100.), (0., 100.));

// uncomment these for stress testing with additional tree depth
//let a = make_overlay(ui.tree(), &root, "a", true);
//let b = make_overlay(ui.tree(), &a, "b", true);
//let c = make_overlay(ui.tree(), &b, "c", true);
//let d = make_overlay(ui.tree(), &c, "d", true);
//let e = make_overlay(ui.tree(), &d, "e", true);

    // spawn 10k buttons
    for x in 0..100
    {
        for y in 0..100
        {
            //spawn_button(&mut commands, &asset_server, &mut ui, &e, x as f32, y as f32);
            spawn_button(&mut ui, &root, x as f32, y as f32);
        }
    }

    // add FPS (upper right corner)
    let fps = relative_widget(ui.tree(), root.end("fps"), (90., 100.), (0., 10.));
    add_fps_section(&mut ui, fps);
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
        .add_plugins(FPSTrackerPlugin)
        .add_plugins(LunexUiPlugin)
        //.add_plugins(UIDebugOverlayPlugin)
        .add_plugins(ReactPlugin)
        .insert_resource(WinitSettings{
            return_from_run : false,
            focused_mode    : UpdateMode::Continuous,  //continuous so we can see FPS
            unfocused_mode  : UpdateMode::ReactiveLowPower{ max_wait: std::time::Duration::from_secs(10) },
        })
        .register_interaction_source(MouseLButtonMain::default())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .add_systems(Last, refresh_fps_indicator)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
