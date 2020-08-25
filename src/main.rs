use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::collide,
};

use rand::distributions::{Distribution, Uniform};

// Colors
const GRASS: Color = Color::rgb(128./255., 191./255., 128./255.);
const ACORN: Color = Color::rgb(128./255., 107./255., 3./255.);
const BUSH: Color = Color::rgb(3./255., 128./255., 78./255.);

// Bounds
const X_MIN: f32 = -450.0;
const X_MAX: f32= 450.0;
const Y_MIN: f32 = -300.0;
const Y_MAX: f32= 300.0;
const WALL_THICKNESS: f32 = 10.0;
const SQUIRREL_THICKNESS: f32 = 48.0;

// Acorns
const NUM_ACORNS: u32 = 5;

fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(GRASS))
        .add_startup_system(setup.system())
        .add_system(squirrel_system.system())
        .add_system(acorn_system.system())
        .run();
}

struct Scoreboard {
    score: usize,
}

struct Squirrel {
    speed: f32,
}

enum Collider {
    Solid,
    Scorable,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Cameras
    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default());

    // Squirrel
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/squirrel.png").unwrap().into()),
            ..Default::default()
        })
        .with(Squirrel{speed: 500.0});
    
    // Title
    commands
        .spawn(TextComponents {
            text: Text {
                font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
                value: "ACORN".to_string(),
                style: TextStyle {
                    color: ACORN,
                    font_size: 60.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });

    // Scoreboard
    commands
        .spawn(TextComponents {
            text: Text {
                font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
                value: "Score: 0".to_string(),
                style: TextStyle {
                    color: ACORN,
                    font_size: 60.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Scoreboard{ score: 0 });

    // Walls
    let wall_material = materials.add(BUSH.into());
    let bounds:Vec2 = Vec2::new(900.0, 600.0);
    commands
        // left
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(-bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(WALL_THICKNESS, bounds.y() + WALL_THICKNESS),
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // right
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(WALL_THICKNESS, bounds.y() + WALL_THICKNESS),
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // bottom
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(0.0, -bounds.y() / 2.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(bounds.x() + WALL_THICKNESS, WALL_THICKNESS),
            },
            ..Default::default()
        })
        .with(Collider::Solid)
        // top
        .spawn(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(0.0, bounds.y() / 2.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(bounds.x() + WALL_THICKNESS, WALL_THICKNESS),
            },
            ..Default::default()
        })
        .with(Collider::Solid);

    // Acorns
    let acorn_size = Vec2::new(20.0, 20.0);
    let mut rng = rand::thread_rng();
    let x_distribution = Uniform::from((X_MIN + WALL_THICKNESS/2.)..(X_MAX - WALL_THICKNESS/2.));
    let y_distribution = Uniform::from((Y_MIN + WALL_THICKNESS/2.)..(Y_MAX - WALL_THICKNESS/2.));
    let mut a = NUM_ACORNS;
    while a > 0 {
        let x_position = x_distribution.sample(&mut rng);
        let y_position = y_distribution.sample(&mut rng);
        let collision = collide(
            Vec3::new(x_position, y_position, 0.), acorn_size, 
            Vec3::new(0., 0., 0.), Vec2::new(SQUIRREL_THICKNESS, SQUIRREL_THICKNESS)
        );
        if let Some(_) = collision {
            continue;
        }

        commands.spawn(SpriteComponents{
            material: materials.add(asset_server.load("assets/textures/acorn.png").unwrap().into()),
            sprite: Sprite { size: acorn_size },
            translation: Translation(Vec3::new(x_position, y_position, 0.)),
            ..Default::default() 
        })
        .with(Collider::Scorable);

        a -= 1;
    } 
}

fn squirrel_system(
    time: Res<Time>, 
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<(&Squirrel, &mut Translation)>,
) {
    for (squirrel, mut translation) in &mut query.iter() {
        let mut x_direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            x_direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            x_direction += 1.0;
        }
        *translation.0.x_mut() = get_new_distance(
            *translation.0.x_mut(),
            time.delta_seconds, x_direction, squirrel.speed,
            X_MIN, X_MAX
        );

        let mut y_direction = 0.0;
        if keyboard_input.pressed(KeyCode::Down) {
            y_direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            y_direction += 1.0;
        }
        *translation.0.y_mut() = get_new_distance(
            *translation.0.y_mut(),
            time.delta_seconds, y_direction, squirrel.speed,
            Y_MIN, Y_MAX
        );
    }
}

fn get_new_distance(
    current_distance: f32,
    delta_time: f32, 
    direction: f32, 
    speed: f32,
    min_bound: f32,
    max_bound: f32,
) -> f32 {
    let new_distance = current_distance + delta_time * direction * speed;

    let thickness = SQUIRREL_THICKNESS + WALL_THICKNESS;
    if new_distance >= (max_bound - thickness/2.) || new_distance <= (min_bound + thickness/2.) {
        return current_distance;
    } else {
        return new_distance;
    }
}

fn acorn_system(
    mut commands: Commands, 
    mut scoreboard_query: Query<(&mut Scoreboard, &mut Text)>, 
    mut squirrel_query: Query<(&Translation, &Sprite)>,
    mut collider_query: Query<(Entity, &Collider, &Translation, &Sprite)>
) {
    for (squirrel_translation, squirrel_sprite) in &mut squirrel_query.iter() {
        for (collider_entity, collider, collider_translation, collider_sprite) in &mut collider_query.iter() {
            let collision = collide(
                squirrel_translation.0, squirrel_sprite.size, 
                collider_translation.0, collider_sprite.size
            );

            if let Some(_) = collision {
                if let Collider::Scorable = *collider {
                    for (mut scoreboard, mut text) in &mut scoreboard_query.iter() {
                        scoreboard.score += 1;
                        text.value = format!("Score: {}", scoreboard.score);
                    }
                    commands.despawn(collider_entity);
                }
                break;
            }
        }
    }
}