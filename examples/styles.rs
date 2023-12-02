//local shortcuts
use bevy_kot::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowTheme};
use bevy_lunex::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Style, Copy, Clone)]
struct StyleA
{
    color: Color,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Style, Copy, Clone)]
struct StyleB
{
    color: Color,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn spawn_box(ui: &mut UiBuilder<MainUi>, area: &Widget, color: Color)
{
    let image = ImageElementBundle::new(
            area,
            ImageParams::center()
                .with_width(Some(100.))
                .with_height(Some(100.))
                .with_color(color),
            ui.asset_server.load("box.png"),
            Vec2::new(236.0, 139.0)
        );
    ui.commands().spawn(image);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn section(ui: &mut UiBuilder<MainUi>, area: &Widget) -> (Widget, Widget)
{
    let (widget_a,_) = ui.div_rel(area.end("a"), (0., 50.), (25., 75.), move |ui, area| {
        spawn_box(ui, area, ui.style::<StyleA>().color);
    });
    let (widget_b,_) = ui.div_rel(area.end("b"), (50., 100.), (25., 75.), move |ui, area| {
        spawn_box(ui, area, ui.style::<StyleB>().color);
    });

    (widget_a, widget_b)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut ui: UiBuilder<MainUi>)
{
    // add base styles
    ui.add_style((StyleA{ color: Color::WHITE }, StyleB{ color: Color::BLACK }));

    // build ui tree
    ui.div_rel("root", (0., 100.), (0., 100.), move |ui, area| {
        let (widget_a, widget_b) = section(ui, area);

        ui.div(move |ui| {
            ui.add_style(StyleA{ color: Color::BLUE });
            section(ui, &widget_a);
        });
        ui.div(move |ui| {
            ui.add_style(StyleA{ color: Color::ORANGE });
            ui.edit_style::<StyleB>(|style| { style.color = Color::GREEN; }).unwrap();
            section(ui, &widget_b);
        });
    });
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
    commands.spawn((Cursor::new(), Transform::default(), Visibility::default()));

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
        .add_plugins(ReactPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
