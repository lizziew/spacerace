use bevy::{
    prelude::*,
    render::pass::ClearColor,
};
use bevy_window::WindowMode;
use rand::distributions::{Distribution, Uniform};
use std::sync::atomic::{AtomicBool, Ordering};

// Colors
const CAVE_COLOR: Color = Color::rgb(192./255., 117./255., 217./255.);
const TEXT_COLOR: Color = Color::rgb(56./255., 41./255., 3./255.);
const WALL_COLOR: Color = Color::rgb(37./255., 3./255., 82./255.);

// Bounds
const WIDTH: f32 = 3000.;
const HEIGHT: f32 = 1600.;
const X_MIN: f32 = -WIDTH/2.;
const X_MAX: f32= WIDTH/2.;
const Y_MIN: f32 = -HEIGHT/2.;
const Y_MAX: f32= HEIGHT/2.;

// Sizes
const WALL_SIZE: f32 = 100.0;
const ASTRONAUT_SIZE: f32 = 40.0;
const ALIEN_SIZE: f32 = 60.0;
const JEWEL_SIZE: f32 = 25.0;
const BASE_SIZE: f32 = 100.0;

// Win/lose
const NUM_JEWELS: u32 = 5;
static GAME_FINISHED: AtomicBool = AtomicBool::new(false);

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "SPACE RACE".to_string(),
            width: WIDTH as u32 + WALL_SIZE as u32,
            height: HEIGHT as u32 + WALL_SIZE as u32,
            vsync: true,
            resizable: false,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .add_default_plugins()
        .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(CAVE_COLOR))
        .add_startup_system(setup.system())
        .add_system(interactions_system.system())
        .run();
}

pub struct Collision {
    x_depth: f32,
    y_depth: f32,
}

struct Object {
    position: Vec3<>,
    size: Vec2<>,
}

struct Scoreboard {
    score: u32,
}
enum Collider {
    Solid,
    Scorable,
    Home,
}

trait Player {
    fn new(speed: f32) -> Self;
    fn speed(&self) -> f32;
}
struct Alien {
    speed: f32,
}

impl Player for Alien {
    fn new(speed: f32) -> Alien {
        Alien { speed }
    }

    fn speed(&self) -> f32 {
        self.speed
    }
}

struct Astronaut {
    speed: f32,
}

impl Player for Astronaut {
    fn new(speed: f32) -> Astronaut {
        Astronaut { speed }
    }

    fn speed(&self) -> f32 {
        self.speed
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_output: Res<AudioOutput>,
) {
    // Cameras
    commands
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default());

    // Astronaut
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/astronaut.png").unwrap().into()),
            translation: Translation::new(X_MIN + WALL_SIZE + ASTRONAUT_SIZE, BASE_SIZE, 0.0),
            ..Default::default()
        })
        .with(Astronaut{ speed: 500.0 });

    // Alien
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/alien.png").unwrap().into()),
            translation: Translation::new(X_MAX - WALL_SIZE - ALIEN_SIZE, 0.0, 0.0),
            ..Default::default()
        })
        .with(Alien{ speed: 500.0 });

    // Home base
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/spaceship.png").unwrap().into()),
            translation: Translation::new(X_MIN + WALL_SIZE + BASE_SIZE, 0.0, 0.0),
            ..Default::default()
        })
        .with(Collider::Home);
    
    // Title
    commands
        .spawn(TextComponents {
            text: Text {
                font: asset_server.load("assets/fonts/raidercrusader.ttf").unwrap(),
                value: "SPACE RACE".to_string(),
                style: TextStyle {
                    color: TEXT_COLOR,
                    font_size: 100.0,
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
                font: asset_server.load("assets/fonts/raidercrusader.ttf").unwrap(),
                value: "SCORE: 0".to_string(),
                style: TextStyle {
                    color: TEXT_COLOR,
                    font_size: 100.0,
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
    let wall_material = materials.add(WALL_COLOR.into());
    commands
    // left
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(X_MIN, 0.0, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WALL_SIZE, HEIGHT + WALL_SIZE),
        },
        ..Default::default()
    })
    .with(Collider::Solid)
    // right
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(X_MAX, 0.0, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WALL_SIZE, HEIGHT + WALL_SIZE),
        },
        ..Default::default()
    })
    .with(Collider::Solid)
    // bottom
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(0.0, Y_MIN, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WIDTH + WALL_SIZE, WALL_SIZE),
        },
        ..Default::default()
    })
    .with(Collider::Solid)
    // top
    .spawn(SpriteComponents {
        material: wall_material,
        translation: Translation(Vec3::new(0.0, Y_MAX, 0.0)),
        sprite: Sprite {
            size: Vec2::new(WIDTH + WALL_SIZE, WALL_SIZE),
        },
        ..Default::default()
    })
    .with(Collider::Solid);

    // Barriers
    let mut rng = rand::thread_rng();
    let barrier_distribution = Uniform::from(1..3); 
    let barrier_columns = (WIDTH / WALL_SIZE) as u32 / 2 - 2;
    let start = X_MIN + 2. * WALL_SIZE;
    for barrier_index in 0..barrier_columns {
        let x = start + (barrier_index as f32 * 2. + 1.) * WALL_SIZE;
        let mut y = Y_MIN + WALL_SIZE;
        while y < Y_MAX - WALL_SIZE {
            if barrier_distribution.sample(&mut rng) == 1 {
                commands
                    .spawn(SpriteComponents {
                        material: materials.add(asset_server.load("assets/textures/wall.png").unwrap().into()),
                        translation: Translation(Vec3::new(x, y, 0.0)),
                        ..Default::default()
                    })
                    .with(Collider::Solid);
            }

            y += WALL_SIZE;
        }
    }

    // Jewels
    let jewel_distribution = Uniform::from(1..10);
    let jewel_columns = barrier_columns - 1;
    for jewel_index in 0..jewel_columns {
        let x = start + (jewel_index as f32 * 2. + 2.) * WALL_SIZE;
        let mut y = Y_MIN + WALL_SIZE;
        while y < Y_MAX - WALL_SIZE {
            if jewel_distribution.sample(&mut rng) == 1 {
                commands.spawn(SpriteComponents{
                    material: materials.add(asset_server.load("assets/textures/jewel.png").unwrap().into()),
                    translation: Translation(Vec3::new(x, y, 0.0)),
                    sprite: Sprite { size: Vec2::new(JEWEL_SIZE, JEWEL_SIZE) },
                    ..Default::default() 
                })
                .with(Collider::Scorable);
            }

            y += WALL_SIZE;
        }
    }

    // Music
    let music = asset_server
        .load("assets/sounds/mii.mp3")
        .unwrap();
    audio_output.play(music);
}

fn interactions_system(
    mut commands: Commands, 
    time: Res<Time>, 
    keyboard_input: Res<Input<KeyCode>>, 
    mut scoreboard_query: Query<(&mut Scoreboard, &mut Text)>, 
    mut astronaut_query: Query<(&Astronaut, &mut Translation, &Sprite)>,
    mut alien_query: Query<(&Alien, &mut Translation, &Sprite)>,
    mut collider_query: Query<(Entity, &Collider, &Translation, &Sprite)>,
) {
    // Get all solid collider objects 
    let mut objects: Vec<Object> = vec![];
    for (_, collider, collider_translation, collider_sprite) in &mut collider_query.iter() {
        if let Collider::Solid = *collider { 
            objects.push(
                Object {
                    position: collider_translation.0,
                    size: collider_sprite.size
                }
            );
        }
    }

    for (astronaut, mut astronaut_translation, astronaut_sprite) in &mut astronaut_query.iter() {
        for (collider_entity, collider, collider_translation, collider_sprite) in &mut collider_query.iter() {
            // Check if astronaut collides with a collider object
            let collision = collide(
                astronaut_translation.0, astronaut_sprite.size, 
                collider_translation.0, collider_sprite.size
            );

            if let Some(_) = collision {
                if let Collider::Scorable = *collider {
                    // Astronaut collides with jewel
                    for (mut scoreboard, mut text) in &mut scoreboard_query.iter() {
                        if !GAME_FINISHED.load(Ordering::Relaxed) {
                            scoreboard.score += 1;
                            text.value = format!("SCORE: {}", scoreboard.score);
                        }
                    }
                    commands.despawn(collider_entity);
                } else if let Collider::Home = *collider {
                    // Astronaut collides with home base
                    for (scoreboard, mut text) in &mut scoreboard_query.iter() {
                        if scoreboard.score >= NUM_JEWELS && !GAME_FINISHED.load(Ordering::Relaxed) {
                            text.value = "ASTRONAUT WINS".to_string();
                            GAME_FINISHED.store(true, Ordering::Relaxed)
                        }
                    }
                }       
                
                break;
            }
        }

        // Astronaut collides with alien
        for (_, alien_translation, alien_sprite) in &mut alien_query.iter() {
            let collision = collide(
                astronaut_translation.0, astronaut_sprite.size, 
                alien_translation.0, alien_sprite.size
            );

            if let Some(_) = collision {       
                for (_, mut text) in &mut scoreboard_query.iter() {
                    if !GAME_FINISHED.load(Ordering::Relaxed) {
                        text.value = "ALIEN WINS".to_string();
                        GAME_FINISHED.store(true, Ordering::Relaxed)
                    }
                }
            }

            break;
        }

        // Move astronaut
        update_position(
            &time,
            &mut astronaut_translation,
            astronaut,
            astronaut_sprite,
            &objects,
            &keyboard_input,
            KeyCode::A, KeyCode::D, KeyCode::S, KeyCode::W
        )
    }

    for (alien, mut alien_translation, alien_sprite) in &mut alien_query.iter() {
        // Move alien
        update_position(
            &time,
            &mut alien_translation,
            alien,
            alien_sprite,
            &objects,
            &keyboard_input,
            KeyCode::Left, KeyCode::Right, KeyCode::Down, KeyCode::Up
        );
    }
}

fn update_position(
    time: &Res<Time>,
    translation: &mut Translation,
    player: &impl Player,
    sprite: &Sprite,
    objects: &Vec<Object>,
    keyboard_input: &Res<Input<KeyCode>>, 
    left: KeyCode,
    right: KeyCode,
    down: KeyCode, 
    up: KeyCode
) {
    let mut x_direction = 0.0;
    if keyboard_input.pressed(left) {
        x_direction -= 1.0;
    }
    if keyboard_input.pressed(right) {
        x_direction += 1.0;
    }
    let new_x_position = get_new_player_position(
        *translation.0.x_mut(),
        time.delta_seconds, x_direction, sprite.size[0], player.speed(),
        X_MIN, X_MAX
    );

    let mut y_direction = 0.0;
    if keyboard_input.pressed(down) {
        y_direction -= 1.0;
    }
    if keyboard_input.pressed(up) {
        y_direction += 1.0;
    }
    let new_y_position = get_new_player_position(
        *translation.0.y_mut(),
        time.delta_seconds, y_direction, sprite.size[1], player.speed(),
        Y_MIN, Y_MAX
    );

    let collision = collides_with_objects(Vec3::new(new_x_position, new_y_position, 0.), sprite.size, &objects);
    if let Some(c) = collision { 
        *translation.0.x_mut() = new_x_position - c.x_depth;
        *translation.0.y_mut() = new_y_position - c.y_depth;
    } else {
        *translation.0.x_mut() = new_x_position;
        *translation.0.y_mut() = new_y_position;
    }
}

pub fn collide(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<Collision> {
    let a_min = a_pos.truncate() - a_size / 2.0;
    let a_max = a_pos.truncate() + a_size / 2.0;

    let b_min = b_pos.truncate() - b_size / 2.0;
    let b_max = b_pos.truncate() + b_size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x() < b_max.x()
        && a_max.x() > b_min.x()
        && a_min.y() < b_max.y()
        && a_max.y() > b_min.y()
    {
        // check to see if we hit on the left or right side
        let (x_collision, x_depth) =
            if a_min.x() < b_min.x() && a_max.x() > b_min.x() && a_max.x() < b_max.x() {
                (true, a_max.x() - b_min.x())
            } else if a_min.x() > b_min.x() && a_min.x() < b_max.x() && a_max.x() > b_max.x() {
                (true, a_min.x() - b_max.x())
            } else {
                (false, 0.0)
            };

        // check to see if we hit on the top or bottom side
        let (y_collision, y_depth) =
            if a_min.y() < b_min.y() && a_max.y() > b_min.y() && a_max.y() < b_max.y() {
                (true, a_max.y() - b_min.y())
            } else if a_min.y() > b_min.y() && a_min.y() < b_max.y() && a_max.y() > b_max.y() {
                (true, a_min.y() - b_max.y())
            } else {
                (false, 0.0)
            };

        if !x_collision && !y_collision {
            return None;
        }

        return Some(Collision{ x_depth, y_depth });
    } else {
        None
    }
}

fn collides_with_objects(
    position: Vec3<>,
    size: Vec2<>,
    objects: &Vec<Object>
) -> Option<Collision> {
    for object in objects {
        let collision = collide(
            position, size,
            object.position, object.size
        );
        if let Some(_) = collision {
            return collision;
        }
    }
    return None;
}

fn get_new_player_position(
    current_position: f32,
    delta_time: f32, 
    direction: f32, 
    size: f32,
    speed: f32,
    min_bound: f32,
    max_bound: f32,
) -> f32 {
    let new_position = current_position + delta_time * direction * speed;

    let buffer = size + WALL_SIZE;
    if new_position >= (max_bound - buffer/2.) || new_position <= (min_bound + buffer/2.) {
        return current_position;
    } else {
        return new_position;
    }
}
