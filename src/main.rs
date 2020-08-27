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
const DOG_THICKNESS: f32 = 48.0;

// Things 
const NUM_ACORNS: u32 = 5;
const NUM_BUSHES: u32 = 20;

// Win/lose
static mut TRIGGERED: bool = false;
static mut WIN: bool = false;

fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(GRASS))
        .add_startup_system(setup.system())
        .add_system(interactions_system.system())
        .run();
}

struct Thing {
    position: Vec3<>,
    size: Vec2<>,
}

struct Scoreboard {
    score: u32,
}

struct Squirrel {
    speed: f32,
}

struct Dog {
    speed: f32,
}

enum Collider {
    Solid,
    Scorable,
    Enemy,
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
        .with(Squirrel{ speed: 500.0 });

    // Dog
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/dog.png").unwrap().into()),
            translation: Translation::new(300.0, 100.0, 0.0),
            ..Default::default()
        })
        .with(Dog{ speed: 40.0 })
        .with(Collider::Enemy)
        .with(Timer::from_seconds(0.05, true));

    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/dog.png").unwrap().into()),
            translation: Translation::new(-200.0, 200.0, 0.0),
            ..Default::default()
        })
        .with(Dog{ speed: 40.0 })
        .with(Collider::Enemy)
        .with(Timer::from_seconds(0.05, true));
    
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

    // Use these to generate random positions
    let mut rng = rand::thread_rng(); 
    let x_distribution = Uniform::from((X_MIN + 20.)..(X_MAX - 20.));
    let y_distribution = Uniform::from((Y_MIN + 20.)..(Y_MAX - 20.));
    let mut existing_things: Vec<Thing> = vec![
        Thing {
            position: Vec3::new(0., 0., 0.),
            size: Vec2::new(SQUIRREL_THICKNESS, SQUIRREL_THICKNESS)
        }
    ];

    // Bushes
    let bush_size = Vec2::new(45., 30.);
    let mut b = NUM_BUSHES;
    while b > 0 {
        let x_position = x_distribution.sample(&mut rng);
        let y_position = y_distribution.sample(&mut rng);
        let bush_position = Vec3::new(x_position, y_position, 0.);

        if collides_with_existing_entity(bush_position, bush_size, &existing_things) {
            continue;
        }

        commands.spawn(SpriteComponents{
            material: materials.add(asset_server.load("assets/textures/bush.png").unwrap().into()),
            sprite: Sprite { size: bush_size },
            translation: Translation(bush_position),
            ..Default::default() 
        })
        .with(Collider::Solid);

        existing_things.push(
            Thing {
                position: bush_position,
                size: bush_size
            }
        );

        b -= 1;
    }

    // Acorns
    let acorn_size = Vec2::new(20.0, 20.0);
    let mut a = NUM_ACORNS;
    while a > 0 {
        let x_position = x_distribution.sample(&mut rng);
        let y_position = y_distribution.sample(&mut rng);
        let acorn_position = Vec3::new(x_position, y_position, 0.);

        if collides_with_existing_entity(acorn_position, acorn_size, &existing_things) {
            continue;
        }

        commands.spawn(SpriteComponents{
            material: materials.add(asset_server.load("assets/textures/acorn.png").unwrap().into()),
            sprite: Sprite { size: acorn_size },
            translation: Translation(acorn_position),
            ..Default::default() 
        })
        .with(Collider::Scorable);

        existing_things.push(
            Thing {
                position: acorn_position,
                size: acorn_size
            }
        );

        a -= 1;
    } 
}

fn collides_with_existing_entity(
    position: Vec3<>,
    size: Vec2<>,
    existing_things: &Vec<Thing>
) -> bool {
    for existing_entity in existing_things {
        let collision = collide(
            position, size,
            existing_entity.position, existing_entity.size
        );
        if let Some(_) = collision {
            return true;
        }
    }
    return false;
}

fn interactions_system(
    mut commands: Commands, 
    time: Res<Time>, 
    keyboard_input: Res<Input<KeyCode>>, 
    mut scoreboard_query: Query<(&mut Scoreboard, &mut Text)>, 
    mut squirrel_query: Query<(&Squirrel, &mut Translation, &Sprite)>,
    mut dog_query: Query<(&mut Timer, &Dog, &mut Translation, &Sprite)>,
    mut collider_query: Query<(Entity, &Collider, &Translation, &Sprite)>
) {
    let mut existing_things: Vec<Thing> = vec![];
    for (_, collider, collider_translation, collider_sprite) in &mut collider_query.iter() {
        if let Collider::Solid = *collider { 
            existing_things.push(
                Thing {
                    position: collider_translation.0,
                    size: collider_sprite.size
                }
            );
        }
    }

    let mut rng = rand::thread_rng();
    let direction_distribution = Uniform::from(-1.0..1.0);
    for (timer, dog, mut dog_translation, dog_sprite) in &mut dog_query.iter() {
        if timer.finished {
            let mut new_x_position;
            let mut new_y_position;
            let mut tries = 5;
            while tries > 0 {
                let x_direction = direction_distribution.sample(&mut rng);
                new_x_position = get_new_dog_position(
                    *dog_translation.0.x_mut(),
                    x_direction,
                    dog.speed,
                    X_MIN,
                    X_MAX
                );

                let y_direction = direction_distribution.sample(&mut rng);
                new_y_position = get_new_dog_position(
                    *dog_translation.0.y_mut(),
                    y_direction,
                    dog.speed,
                    Y_MIN,
                    Y_MAX
                );

                if !collides_with_existing_entity(Vec3::new(new_x_position, new_y_position, 0.), dog_sprite.size, &existing_things) {
                    *dog_translation.0.x_mut() = new_x_position;
                    *dog_translation.0.y_mut() = new_y_position;
                    break;
                } 

                tries -= 1;
            }
        }
    }

    for (squirrel, mut squirrel_translation, squirrel_sprite) in &mut squirrel_query.iter() {
        for (collider_entity, collider, collider_translation, collider_sprite) in &mut collider_query.iter() {
            let collision = collide(
                squirrel_translation.0, squirrel_sprite.size, 
                collider_translation.0, collider_sprite.size
            );

            if let Some(_) = collision {
                if let Collider::Scorable = *collider {
                    for (mut scoreboard, mut text) in &mut scoreboard_query.iter() {
                        unsafe {
                            if !TRIGGERED {
                                scoreboard.score += 1;
                                text.value = format!("Score: {}", scoreboard.score);
                            }
                        }
                    }
                    commands.despawn(collider_entity);
                }          
                
                if let Collider::Enemy = *collider {
                    for (_, mut text) in &mut scoreboard_query.iter() {
                        unsafe {
                            if !TRIGGERED {
                                text.value = "YOU LOSE :(".to_string();
                                TRIGGERED = true;
                                WIN = false;
                            }
                        }
                    }
                }
                break;
            }
        }

        let mut x_direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            x_direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            x_direction += 1.0;
        }
        let new_x_position = get_new_squirrel_position(
            *squirrel_translation.0.x_mut(),
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
        let new_y_position = get_new_squirrel_position(
            *squirrel_translation.0.y_mut(),
            time.delta_seconds, y_direction, squirrel.speed,
            Y_MIN, Y_MAX
        );

        if collides_with_existing_entity(Vec3::new(new_x_position, new_y_position, 0.), squirrel_sprite.size, &existing_things) {
            continue;
        }

        *squirrel_translation.0.x_mut() = new_x_position;
        *squirrel_translation.0.y_mut() = new_y_position;
    }

    for (scoreboard, mut text) in &mut scoreboard_query.iter() {
        if scoreboard.score == NUM_ACORNS {
            unsafe {
                if !TRIGGERED {
                    text.value = "YOU WIN!".to_string();
                    TRIGGERED = true;
                    WIN = true;
                }
            }
        }
    }
}

fn get_new_dog_position(
    current_position: f32,
    direction: f32, 
    speed: f32,
    min_bound: f32,
    max_bound: f32,
) -> f32 {
    let new_position = current_position + direction * speed;

    let thickness = DOG_THICKNESS + WALL_THICKNESS;
    if new_position >= (max_bound - thickness/2.) || new_position <= (min_bound + thickness/2.) {
        return current_position;
    } else {
        return new_position;
    }
}


fn get_new_squirrel_position(
    current_position: f32,
    delta_time: f32, 
    direction: f32, 
    speed: f32,
    min_bound: f32,
    max_bound: f32,
) -> f32 {
    let new_position = current_position + delta_time * direction * speed;

    let thickness = SQUIRREL_THICKNESS + WALL_THICKNESS;
    if new_position >= (max_bound - thickness/2.) || new_position <= (min_bound + thickness/2.) {
        return current_position;
    } else {
        return new_position;
    }
}
