use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::collide,
};
use bevy_window::WindowMode;
use rand::distributions::{Distribution, Uniform};

// Colors
const GRASS: Color = Color::rgb(192./255., 117./255., 217./255.);
const TEXT: Color = Color::rgb(56./255., 41./255., 3./255.);
const BUSH: Color = Color::rgb(37./255., 3./255., 82./255.);

// Bounds
const WIDTH: f32 = 3000.;
const HEIGHT: f32 = 1600.;
const X_MIN: f32 = -WIDTH/2.;
const X_MAX: f32= WIDTH/2.;
const Y_MIN: f32 = -HEIGHT/2.;
const Y_MAX: f32= HEIGHT/2.;

// Sizes
const WALL_SIZE: f32 = 100.0;
const SQUIRREL_SIZE: f32 = 40.0;
const DOG_SIZE: f32 = 60.0;
const ACORN_SIZE: f32 = 25.0;
const HOME_SIZE: f32 = 100.0;

// Win/lose
const NUM_ACORNS: u32 = 5;
static mut TRIGGERED: bool = false;

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
        .add_resource(ClearColor(GRASS))
        .add_startup_system(setup.system())
        .add_system(interactions_system.system())
        .run();
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
struct Dog {
    speed: f32,
}

impl Player for Dog {
    fn new(speed: f32) -> Dog {
        Dog { speed }
    }

    fn speed(&self) -> f32 {
        self.speed
    }
}

struct Squirrel {
    speed: f32,
}

impl Player for Squirrel {
    fn new(speed: f32) -> Squirrel {
        Squirrel { speed }
    }

    fn speed(&self) -> f32 {
        self.speed
    }
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
            material: materials.add(asset_server.load("assets/textures/astronaut.png").unwrap().into()),
            translation: Translation::new(X_MIN + WALL_SIZE + SQUIRREL_SIZE, HOME_SIZE, 0.0),
            ..Default::default()
        })
        .with(Squirrel{ speed: 500.0 });

    // Dog
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/alien.png").unwrap().into()),
            translation: Translation::new(X_MAX - WALL_SIZE - DOG_SIZE, 0.0, 0.0),
            ..Default::default()
        })
        .with(Dog{ speed: 500.0 });

    // Squirrel home
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/textures/spaceship.png").unwrap().into()),
            translation: Translation::new(X_MIN + WALL_SIZE + HOME_SIZE, 0.0, 0.0),
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
                    color: TEXT,
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
                    color: TEXT,
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
    let wall_material = materials.add(BUSH.into());
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

    // Acorns
    let acorn_distribution = Uniform::from(1..10);
    let acorn_columns = barrier_columns - 1;
    for acorn_index in 0..acorn_columns {
        let x = start + (acorn_index as f32 * 2. + 2.) * WALL_SIZE;
        let mut y = Y_MIN + WALL_SIZE;
        while y < Y_MAX - WALL_SIZE {
            if acorn_distribution.sample(&mut rng) == 1 {
                commands.spawn(SpriteComponents{
                    material: materials.add(asset_server.load("assets/textures/jewel.png").unwrap().into()),
                    translation: Translation(Vec3::new(x, y, 0.0)),
                    sprite: Sprite { size: Vec2::new(ACORN_SIZE, ACORN_SIZE) },
                    ..Default::default() 
                })
                .with(Collider::Scorable);
            }

            y += WALL_SIZE;
        }
    }
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

    for (squirrel, mut squirrel_translation, squirrel_sprite) in &mut squirrel_query.iter() {
        for (collider_entity, collider, collider_translation, collider_sprite) in &mut collider_query.iter() {
            let collision = collide(
                squirrel_translation.0, squirrel_sprite.size, 
                collider_translation.0, collider_sprite.size
            );

            if let Some(_) = collision {
                if let Collider::Scorable = *collider {
                    // Squirrel collides with acorn
                    for (mut scoreboard, mut text) in &mut scoreboard_query.iter() {
                        unsafe {
                            if !TRIGGERED {
                                scoreboard.score += 1;
                                text.value = format!("SCORE: {}", scoreboard.score);
                            }
                        }
                    }
                    commands.despawn(collider_entity);
                } else if let Collider::Home = *collider {
                    // Squirrel collides with home
                    for (scoreboard, mut text) in &mut scoreboard_query.iter() {
                        unsafe {
                            if scoreboard.score >= NUM_ACORNS && !TRIGGERED {
                                text.value = "ASTRONAUT WINS".to_string();
                                TRIGGERED = true;
                            }
                        }
                    }
                }       
                
                break;
            }
        }

        // Squirrel collides with dog
        for (_, dog_translation, dog_sprite) in &mut dog_query.iter() {
            let collision = collide(
                squirrel_translation.0, squirrel_sprite.size, 
                dog_translation.0, dog_sprite.size
            );

            if let Some(_) = collision {       
                for (_, mut text) in &mut scoreboard_query.iter() {
                    unsafe {
                        if !TRIGGERED {
                            text.value = "ALIEN WINS".to_string();
                            TRIGGERED = true;
                        }
                    }
                }
            }

            break;
        }

        // Move squirrel
        update_position(
            &time,
            &mut squirrel_translation,
            squirrel,
            squirrel_sprite,
            &objects,
            &keyboard_input,
            KeyCode::A, KeyCode::D, KeyCode::S, KeyCode::W
        )
    }

    for (dog, mut dog_translation, dog_sprite) in &mut dog_query.iter() {
        // Move dog
        update_position(
            &time,
            &mut dog_translation,
            dog,
            dog_sprite,
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

    if !collides_with_objects(Vec3::new(new_x_position, new_y_position, 0.), sprite.size, &objects) {
        *translation.0.x_mut() = new_x_position;
        *translation.0.y_mut() = new_y_position;
    }
}

fn collides_with_objects(
    position: Vec3<>,
    size: Vec2<>,
    objects: &Vec<Object>
) -> bool {
    for object in objects {
        let collision = collide(
            position, size,
            object.position, object.size
        );
        if let Some(_) = collision {
            return true;
        }
    }
    return false;
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
