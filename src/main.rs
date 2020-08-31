use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::collide,
};
use bevy_window::WindowMode;
use rand::distributions::{Distribution, Uniform};

// Colors
const GRASS: Color = Color::rgb(128./255., 191./255., 128./255.);
const ACORN: Color = Color::rgb(128./255., 107./255., 3./255.);
const BUSH: Color = Color::rgb(3./255., 128./255., 78./255.);

// Bounds
const WIDTH: f32 = 2000.;
const HEIGHT: f32 = 1200.;
const X_MIN: f32 = -WIDTH/2.;
const X_MAX: f32= WIDTH/2.;
const Y_MIN: f32 = -HEIGHT/2.;
const Y_MAX: f32= HEIGHT/2.;
const WALL_THICKNESS: f32 = 100.0;
const SQUIRREL_SIZE: f32 = 48.0;
const DOG_THICKNESS: f32 = 48.0;
const ACORN_SIZE: f32 = 20.0;
const HOME_WIDTH: f32 = 80.;
const HOME_HEIGHT: f32 = 100.;

// Things 
const NUM_ACORNS: u32 = 5;

// Win/lose
static mut TRIGGERED: bool = false;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "acorn".to_string(),
            width: WIDTH as u32 + WALL_THICKNESS as u32,
            height: HEIGHT as u32 + WALL_THICKNESS as u32,
            vsync: true,
            resizable: false,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
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
    Home,
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
            translation: Translation::new(X_MIN + WALL_THICKNESS + SQUIRREL_SIZE, HOME_HEIGHT, 0.0),
            ..Default::default()
        })
        .with(Squirrel{ speed: 500.0 });

    // Dog
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/dog.png").unwrap().into()),
            translation: Translation::new(X_MAX - WALL_THICKNESS - DOG_THICKNESS, 0.0, 0.0),
            ..Default::default()
        })
        .with(Dog{ speed: 500.0 });

    // Squirrel home
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/home.png").unwrap().into()),
            translation: Translation::new(X_MIN + WALL_THICKNESS + HOME_WIDTH, 0.0, 0.0),
            ..Default::default()
        })
        .with(Collider::Home);
    
    // Title
    commands
        .spawn(TextComponents {
            text: Text {
                font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
                value: "ACORN".to_string(),
                style: TextStyle {
                    color: ACORN,
                    font_size: 50.0,
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
                    font_size: 50.0,
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
    commands
    // left
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(X_MIN, 0.0, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WALL_THICKNESS, HEIGHT + WALL_THICKNESS),
        },
        ..Default::default()
    })
    .with(Collider::Solid)
    // right
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(X_MAX, 0.0, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WALL_THICKNESS, HEIGHT + WALL_THICKNESS),
        },
        ..Default::default()
    })
    .with(Collider::Solid)
    // bottom
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(0.0, Y_MIN, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WIDTH + WALL_THICKNESS, WALL_THICKNESS),
        },
        ..Default::default()
    })
    .with(Collider::Solid)
    // top
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(0.0, Y_MAX, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WIDTH + WALL_THICKNESS, WALL_THICKNESS),
        },
        ..Default::default()
    })
    .with(Collider::Solid);

    // Barriers
    let mut rng = rand::thread_rng();
    let barrier_distribution = Uniform::from(1..3); 
    let number_of_walls = (WIDTH / WALL_THICKNESS) as u32 / 2 - 2;
    let start = X_MIN + 2. * WALL_THICKNESS;
    for wall_index in 0..number_of_walls {
        let x = start + (wall_index as f32 * 2. + 1.) * WALL_THICKNESS;
        let mut y = Y_MIN + WALL_THICKNESS / 2.;
        while y < Y_MAX {
            if barrier_distribution.sample(&mut rng) == 1 {
                commands
                    .spawn(SpriteComponents {
                        material: wall_material,
                        translation: Translation(Vec3::new(x, y, 0.0)),
                        sprite: Sprite {
                            size: Vec2::new(WALL_THICKNESS, WALL_THICKNESS),
                        },
                        ..Default::default()
                    })
                    .with(Collider::Solid);
            }

            y += WALL_THICKNESS;
        }
    }

    // Acorns
    let acorn_distribution = Uniform::from(1..5);
    let acorn_columns = number_of_walls - 1;
    for acorn_index in 0..acorn_columns {
        let x = start + (acorn_index as f32 * 2. + 2.) * WALL_THICKNESS;
        let mut y = Y_MIN + WALL_THICKNESS;
        while y < Y_MAX - WALL_THICKNESS {
            if acorn_distribution.sample(&mut rng) == 1 {
                commands.spawn(SpriteComponents{
                    material: materials.add(asset_server.load("assets/textures/acorn.png").unwrap().into()),
                    translation: Translation(Vec3::new(x, y, 0.0)),
                    sprite: Sprite { size: Vec2::new(ACORN_SIZE, ACORN_SIZE) },
                    ..Default::default() 
                })
                .with(Collider::Scorable);
            }

            y += WALL_THICKNESS;
        }
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
    mut dog_query: Query<(&Dog, &mut Translation, &Sprite)>,
    mut collider_query: Query<(Entity, &Collider, &Translation, &Sprite)>,
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
                } else if let Collider::Home = *collider {
                    for (scoreboard, mut text) in &mut scoreboard_query.iter() {
                        unsafe {
                            if scoreboard.score >= NUM_ACORNS && !TRIGGERED {
                                text.value = "SQUIRREL WINS".to_string();
                                TRIGGERED = true;
                            }
                        }
                    }
                }       
                
                break;
            }
        }

        for (_, dog_translation, dog_sprite) in &mut dog_query.iter() {
            let collision = collide(
                squirrel_translation.0, squirrel_sprite.size, 
                dog_translation.0, dog_sprite.size
            );

            if let Some(_) = collision {       
                for (_, mut text) in &mut scoreboard_query.iter() {
                    unsafe {
                        if !TRIGGERED {
                            text.value = "DOG WINS".to_string();
                            TRIGGERED = true;
                        }
                    }
                }
            }

            break;
        }

        let mut x_direction = 0.0;
        if keyboard_input.pressed(KeyCode::A) {
            x_direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            x_direction += 1.0;
        }
        let new_x_position = get_new_squirrel_position(
            *squirrel_translation.0.x_mut(),
            time.delta_seconds, x_direction, squirrel.speed,
            X_MIN, X_MAX
        );

        let mut y_direction = 0.0;
        if keyboard_input.pressed(KeyCode::S) {
            y_direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::W) {
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

    for (dog, mut dog_translation, dog_sprite) in &mut dog_query.iter() {
        let mut x_direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            x_direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            x_direction += 1.0;
        }
        let new_x_position = get_new_squirrel_position(
            *dog_translation.0.x_mut(),
            time.delta_seconds, x_direction, dog.speed,
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
            *dog_translation.0.y_mut(),
            time.delta_seconds, y_direction, dog.speed,
            Y_MIN, Y_MAX
        );

        if collides_with_existing_entity(Vec3::new(new_x_position, new_y_position, 0.), dog_sprite.size, &existing_things) {
            continue;
        }

        *dog_translation.0.x_mut() = new_x_position;
        *dog_translation.0.y_mut() = new_y_position;
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

    let thickness = SQUIRREL_SIZE + WALL_THICKNESS;
    if new_position >= (max_bound - thickness/2.) || new_position <= (min_bound + thickness/2.) {
        return current_position;
    } else {
        return new_position;
    }
}
