//path aliases
use bevy_kot::prelude as kot;
use bevy_lunex::prelude as lunex;

//local shortcuts
use kot::Style;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Style)]
struct StyleA
{
    color: Color,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Style)]
struct StyleB
{
    color: Color,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_widget_a(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    style_stack  : &mut kot::StyleStack,
    area         : &lunex::Widget,
) -> lunex::Widget
{
    let widget = lunex::Widget::create(
            ui,
            area.end(""),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 0.0, y: 25.0 },
                relative_2 : Vec2 { x: 50.0, y: 75.0 },
                ..Default::default()
            }
        ).unwrap();

    commands.spawn(
            lunex::ImageElementBundle::new(
                    &widget,
                    lunex::ImageParams::center()
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(style_stack.get::<StyleA>().unwrap().color),
                    asset_server.load("box.png"),
                    Vec2::new(236.0, 139.0)
                )
        );

    widget
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_widget_b(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    style_stack  : &mut kot::StyleStack,
    area         : &lunex::Widget,
) -> lunex::Widget
{
    let widget = lunex::Widget::create(
            ui,
            area.end(""),
            lunex::RelativeLayout{
                relative_1 : Vec2 { x: 50.0, y: 25.0 },
                relative_2 : Vec2 { x: 100.0, y: 75.0 },
                ..Default::default()
            }
        ).unwrap();

    commands.spawn(
            lunex::ImageElementBundle::new(
                    &widget,
                    lunex::ImageParams::center()
                        .with_width(Some(100.))
                        .with_height(Some(100.))
                        .with_color(style_stack.get::<StyleB>().unwrap().color),
                    asset_server.load("box.png"),
                    Vec2::new(236.0, 139.0)
                )
        );

    widget
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_widget_a_subwidgets(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    style_stack  : &mut kot::StyleStack,
    area         : lunex::Widget,
){
    style_stack.add(StyleA{ color: Color::BLUE });
    add_widget_a(commands, asset_server, ui, style_stack, &area);
    add_widget_b(commands, asset_server, ui, style_stack, &area);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_widget_b_subwidgets(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    style_stack  : &mut kot::StyleStack,
    area         : lunex::Widget,
){
    style_stack.add(StyleA{ color: Color::ORANGE });
    style_stack.add(StyleB{ color: Color::GREEN });
    add_widget_a(commands, asset_server, ui, style_stack, &area);
    add_widget_b(commands, asset_server, ui, style_stack, &area);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(
    commands     : &mut Commands,
    asset_server : &AssetServer,
    ui           : &mut lunex::UiTree,
    style_stack  : &mut kot::StyleStack,
    area         : lunex::Widget,
){
    // add widget A
    let widget_a = add_widget_a(commands, asset_server, ui, style_stack, &area);

    // add widgets within widget A
    style_stack.push();
    add_widget_a_subwidgets(commands, asset_server, ui, style_stack, widget_a);
    style_stack.pop();

    // add widget B
    let widget_b = add_widget_b(commands, asset_server, ui, style_stack, &area);

    // add widgets within widget B
    style_stack.push();
    add_widget_b_subwidgets(commands, asset_server, ui, style_stack, widget_b);
    style_stack.pop();
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
    commands.spawn((lunex::Cursor::new(0.0), Transform::default(), kot::builtin::MainMouseCursor));

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

    // prep style stack
    let mut style_stack = kot::StyleStack::default();

    // add style stack frame
    style_stack.push();

    // add base styles
    style_stack.add((StyleA{ color: Color::WHITE }, StyleB{ color: Color::BLACK }));

    // build ui tree
    build_ui(&mut commands, &asset_server, &mut ui, &mut style_stack, root);

    // remove style stack frame
    style_stack.pop();

    // add ui tree to ecs
    commands.spawn(ui);
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
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
