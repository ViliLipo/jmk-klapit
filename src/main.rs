use comfy::*;

comfy_game!("JMK Klapit", KlapiGame, config);

pub fn round_to_precision(number: f32, precision: i32) -> f32 {
    let val: f32 = number * 10.0_f32.powi(precision);
    let rounded_val = val.round();
    return rounded_val / 10.0_f32.powi(precision);
}

fn config(config: GameConfig) -> GameConfig {
    let mut conf = GameConfig {
        vsync_enabled: false,
        target_framerate: 165,
        ..config
    };
    conf.dev.show_fps = true;
    return conf;
}

pub trait GameObject<T> {
    fn update(&self, delta: f32) -> T;
}

pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
}

impl Line {
    pub fn collide(&self, other: &Line) -> bool {
        let denominator = ((self.end.x - self.start.x) * (other.end.y - other.start.y))
            - ((self.end.y - self.start.y) * (other.end.x - other.start.x));
        let numerator1 = ((self.start.y - other.start.y) * (other.end.x - other.start.x))
            - ((self.start.x - other.start.x) * (other.end.y - other.start.y));
        let numerator2 = ((self.start.y - other.start.y) * (self.end.x - self.start.x))
            - ((self.start.x - other.start.x) * (self.end.y - self.start.y));
        if denominator == 0.0 {
            return numerator1 == 0.0 && numerator2 == 0.0;
        }
        let r = numerator1 / denominator;
        let s = numerator2 / denominator;
        return (r >= 0.0 && r <= 1.0) && (s >= 0.0 && s <= 1.0);
    }
}

pub struct Polygon {
    pub vertices: Vec<Line>,
}

impl Polygon {
    pub fn collide_point(&self, point: Vec2) -> bool {
        let mut collision = false;
        for vertex in &self.vertices {
            let val = ((vertex.start.y > point.y && vertex.end.y < point.y)
                || (vertex.start.y < point.y && vertex.end.y > point.y))
                && (point.x
                    < (vertex.end.x - vertex.start.x) * (point.y - vertex.start.y)
                        / (vertex.end.y - vertex.start.y)
                        + vertex.start.x);
            if val {
                collision = !collision
            }
        }
        collision
    }
    pub fn collide_line(&self, line: &Line) -> bool {
        for vertice in &self.vertices {
            if line.collide(vertice) {
                return true;
            }
        }
        return false;
    }

    pub fn collide(&self, other: &Polygon) -> bool {
        for vertice in &self.vertices {
            let mut collision = other.collide_line(vertice);
            if collision {
                return true;
            }
            collision = match other.vertices.first() {
                Some(other_first) => self.collide_point(other_first.start),
                None => false,
            };
            if collision {
                return true;
            }
        }
        return false;
    }
}

#[derive(Debug)]
pub struct Rectangle {
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

impl Rectangle {
    pub fn to_poly(&self) -> Polygon {
        let tl = self.top_left();
        let tr = self.top_right();
        let br = self.bottom_right();
        let bl = self.bottom_left();
        return Polygon {
            vertices: vec![
                Line { start: tl, end: tr },
                Line { start: tr, end: br },
                Line { start: br, end: bl },
                Line { start: bl, end: tl },
            ],
        };
    }

    pub fn collide(&self, rect: &Rectangle) -> bool {
        return self.to_poly().collide(&rect.to_poly());
    }

    pub fn top_left(&self) -> Vec2 {
        let theta = self.rotation.to_radians();
        let x = self.position.x
            - ((self.width * 0.5) * theta.cos())
            - ((self.height * 0.5) * theta.sin());
        let y = self.position.y - ((self.width * 0.5) * theta.sin())
            + ((self.height * 0.5) * theta.cos());
        return vec2(x, y);
    }

    pub fn top_right(&self) -> Vec2 {
        let theta = self.rotation.to_radians();
        let x = self.position.x + ((self.width * 0.5) * theta.cos())
            - ((self.height * 0.5) * theta.sin());
        let y = self.position.y
            + ((self.width * 0.5) * theta.sin())
            + ((self.height * 0.5) * theta.cos());
        return vec2(x, y);
    }

    pub fn bottom_right(&self) -> Vec2 {
        let theta = self.rotation.to_radians();
        let x = self.position.x
            + ((self.width * 0.5) * theta.cos())
            + ((self.height * 0.5) * theta.sin());
        let y = self.position.y + ((self.width * 0.5) * theta.sin())
            - ((self.height * 0.5) * theta.cos());
        return vec2(x, y);
    }

    pub fn bottom_left(&self) -> Vec2 {
        let theta = self.rotation.to_radians();
        let x = self.position.x - ((self.width * 0.5) * theta.cos())
            + ((self.height * 0.5) * theta.sin());
        let y = self.position.y
            - ((self.width * 0.5) * theta.sin())
            - ((self.height * 0.5) * theta.cos());
        return vec2(x, y);
    }

    pub fn pivot(&self, pivot_point: Vec2, angle: f32) -> Rectangle {
        let theta = angle.to_radians();
        let centered_x = self.position.x - pivot_point.x;
        let centered_y = self.position.y - pivot_point.y;
        let d_x = theta.cos() * centered_x - theta.sin() * centered_y;
        let d_y = theta.sin() * centered_x + theta.cos() * centered_y;
        let pos_x = pivot_point.x + d_x;
        let pos_y = pivot_point.y + d_y;
        let position = vec2(round_to_precision(pos_x, 6), round_to_precision(pos_y, 6));
        return Rectangle {
            position,
            width: self.width,
            height: self.height,
            rotation: angle,
        };
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        self.to_poly().collide_point(point)
    }
}

impl Clone for Rectangle {
    fn clone(&self) -> Self {
        Rectangle {
            position: self.position,
            width: self.width,
            height: self.height,
            rotation: self.rotation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Klapi {
    pub rect: Rectangle,
    pub mass: f32,
    pub forces: Vec<Vec2>,
    pub speed: Vec2,
    pub rotational_speed: f32,
    pub max_speed: f32,
}

impl Klapi {
    fn calculate_new_speed(&self, delta: f32) -> Vec2 {
        let mut x_force_sum = 0.0;
        let mut y_force_sum = 0.0;
        for f in &self.forces {
            x_force_sum = x_force_sum + f.x;
            y_force_sum = y_force_sum + f.y;
        }
        let x_change = x_force_sum * (delta / self.mass);
        let y_change = y_force_sum * (delta / self.mass);
        let new_x_speed = (self.speed.x + x_change).min(self.max_speed);
        let new_y_speed = (self.speed.y + y_change).min(self.max_speed);
        return vec2(new_x_speed, new_y_speed);
    }

    fn calculate_new_rect(&self, speed: Vec2, delta: f32) -> Rectangle {
        let x = self.rect.position.x + speed.x * delta;
        let y = self.rect.position.y + speed.y * delta;
        let rotation = self.rect.rotation + self.rotational_speed * delta;
        let position = vec2(x, y);
        return Rectangle {
            height: self.rect.height,
            width: self.rect.width,
            rotation,
            position,
        };
    }
}

impl GameObject<Klapi> for Klapi {
    fn update(&self, delta: f32) -> Klapi {
        let speed = self.calculate_new_speed(delta);
        let rect = self.calculate_new_rect(speed, delta);
        return Klapi {
            rect,
            speed,
            rotational_speed: self.rotational_speed,
            forces: self.forces.clone(),
            max_speed: self.max_speed,
            mass: self.mass,
        };
    }
}

#[derive(Clone)]
pub struct Arm {
    pub arm_rect: Rectangle,
    pub hand_rect: Rectangle,
    pub min_angle: f32,
    pub max_angle: f32,
    pub angle: f32,
    pub radius: f32,
    pub thrown: bool,
    pub acceleration: f32,
    pub speed: f32,
    pub start_location: Vec2,
    pub pivot_location: Vec2,
}

impl Arm {
    fn get_arm_start_rect(&self) -> Rectangle {
        let mut rect = self.arm_rect.clone();
        rect.position = self.start_location.clone();
        return rect;
    }

    fn launch_klapi(&self) -> Klapi {
        let theta = self.angle.to_radians();
        let frequency = 1.0 / (360.0 / self.speed);
        let velocity = 2.0 * PI * frequency * self.radius;
        let x_speed = velocity * theta.cos();
        let y_speed = 1.0 * velocity * theta.sin();
        Klapi {
            rect: Rectangle {
                position: self.arm_rect.bottom_right(),
                height: 0.15,
                width: 0.45,
                rotation: self.angle,
            },
            speed: vec2(x_speed, y_speed),
            rotational_speed: 0.0,
            max_speed: 10.0,
            mass: 2.5,
            forces: vec![vec2(0.0, -9.81 * 2.5)],
        }
    }
}

impl GameObject<Arm> for Arm {
    fn update(&self, delta: f32) -> Arm {
        let speed = self.speed + self.acceleration * delta;
        let mut angle = self.angle + speed * delta;
        if angle > self.max_angle {
            angle = self.max_angle;
        } else if angle < self.min_angle {
            angle = self.min_angle
        }
        let rect = self.get_arm_start_rect();
        let arm_rect = rect.pivot(self.pivot_location, angle);

        return Arm {
            start_location: self.start_location,
            pivot_location: self.pivot_location,
            acceleration: self.acceleration,
            arm_rect,
            angle,
            max_angle: self.max_angle,
            min_angle: self.min_angle,
            speed,
            thrown: self.thrown,
            radius: self.radius,
            hand_rect: self.hand_rect.clone(),
        };
    }
}

fn new_arm(start_location: Vec2) -> Arm {
    let height = 0.9;
    let width = 0.18;
    let angle = 0.0;
    let hand_y = start_location.y - height * 0.5;
    let hand_position = vec2(start_location.x, hand_y);
    let pivot_y = start_location.y + height * 0.5;
    let pivot_location = vec2(start_location.x, pivot_y);
    return Arm {
        start_location,
        pivot_location,
        acceleration: 0.0,
        arm_rect: Rectangle {
            height,
            width,
            position: start_location,
            rotation: angle,
        },
        angle,
        max_angle: 90.0,
        min_angle: -90.0,
        speed: 720.0,
        thrown: false,
        radius: height,
        hand_rect: Rectangle {
            position: hand_position,
            height: width,
            width,
            rotation: angle,
        },
    };
}

#[derive(Clone)]
pub struct Barrier {
    pub bounciness: f32,
    pub rect: Rectangle,
}

impl Barrier {
    fn on_collision(&self, klapi: &Klapi) -> Klapi {
        let rect = self.move_klapi_out_of(klapi);
        let velo = (klapi.speed.x.powf(2.0) + klapi.speed.x.powf(2.0)).sqrt();
        let x_speed = self.bounciness * velo * (0.0_f32).cos();
        let y_speed = self.bounciness * velo * (0.0_f32).sin();
        let speed = vec2(x_speed, y_speed);
        return Klapi {
            forces: klapi.forces.clone(),
            rotational_speed: klapi.rotational_speed,
            speed,
            rect,
            mass: klapi.mass,
            max_speed: klapi.max_speed,
        };
    }

    fn move_klapi_out_of(&self, klapi: &Klapi) -> Rectangle {
        let resolution = 0.01;
        let mut rect = klapi.rect.clone();
        while self.rect.collide(&rect) {
            let old_pos = klapi.rect.position;
            let sign_x = if klapi.speed.x >= 0.0 {
                1.0
            } else {
                - 1.0
            };
            let sign_y = if klapi.speed.y >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let x_delta = if (klapi.speed.x * resolution).abs() <= resolution {
                sign_x * resolution
            } else {
                klapi.speed.x * resolution
            };
            let y_delta = if (klapi.speed.y * resolution).abs() <= resolution {
                sign_y * resolution
            } else {
                klapi.speed.y * resolution
            };
            let x = old_pos.x - x_delta;
            let y = old_pos.y - y_delta;
            rect = Rectangle {
                position: vec2(x, y),
                width: klapi.rect.width,
                height: klapi.rect.height,
                rotation: klapi.rect.rotation,
            };
        }
        return rect;
    }
}

#[derive(Clone)]
pub struct Kiuas {
    pub barriers: Vec<Barrier>,
    pub goal: Rectangle,
}

pub enum GamePhase {
    Start(Arm),
    Charging(Arm),
    Launching(Arm),
    Launched(Arm, Klapi, Kiuas, Vec<Barrier>),
}

fn draw_arm(arm: &Arm) {
    draw_sprite_rot(
        texture_id("arm"),
        arm.arm_rect.position,
        WHITE.alpha(1.0),
        5,
        arm.angle.to_radians(),
        vec2(arm.arm_rect.width, arm.arm_rect.height),
    );
}

fn load_textures(context: &mut EngineContext) {
    context.load_texture_from_bytes(
        "arm",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/assets/jmkhand.png"
        )),
    );
    context.load_texture_from_bytes(
        "body",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/assets/kuvajmk.png"
        )),
    );
    context.load_texture_from_bytes(
        "kiuas",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets/kiuas.png")),
    );
    context.load_texture_from_bytes(
        "background",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/assets/background.png"
        )),
    );
    context.load_texture_from_bytes(
        "klapi",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/assets/klapi.png"
        )),
    );
}

fn draw_statics(origin: Vec2) {
    draw_sprite(
        texture_id("background"),
        origin.clone(),
        WHITE,
        1,
        vec2(8.0, 3.5),
    );
    let body_loc = vec2(origin.x - 2.0, origin.y - 0.25);
    draw_sprite(texture_id("body"), body_loc, WHITE, 2, vec2(1.108, 1.8));
    let kiuas_loc = vec2(origin.x + 3.3, origin.y - 0.1);
    draw_sprite(texture_id("kiuas"), kiuas_loc, WHITE, 2, vec2(1.35, 1.8));
}

pub struct KlapiGame {
    pub phase: GamePhase,
    pub score: u32,
    pub textures_loaded: bool,
}

impl GameLoop for KlapiGame {
    fn new(_es: &mut EngineState) -> Self {
        let mut camera = main_camera_mut();
        camera.zoom = 7.5;
        camera.center = vec2(0.0, 0.0);
        Self {
            score: 0,
            textures_loaded: false,
            phase: GamePhase::Start(new_arm(vec2(-1.9, -0.45))),
        }
    }

    fn update(&mut self, context: &mut EngineContext) {
        if !self.textures_loaded {
            load_textures(context);
            self.textures_loaded = true;
        }
        draw_statics(vec2(0.0, 0.0));
        let time_delta = delta();
        self.phase = match &self.phase {
            GamePhase::Start(arm) => {
                draw_arm(arm);
                if is_key_pressed(KeyCode::Space) {
                    let mut new_arm = arm.clone();
                    new_arm.speed = -60.0;
                    GamePhase::Charging(new_arm)
                } else {
                    GamePhase::Start(arm.clone())
                }
            }
            GamePhase::Charging(arm) => {
                draw_arm(arm);
                if is_key_pressed(KeyCode::Space) || arm.angle <= arm.min_angle {
                    GamePhase::Launching(Arm {
                        speed: 0.0,
                        acceleration: 720.0,
                        ..arm.clone()
                    })
                } else {
                    GamePhase::Charging(arm.update(time_delta))
                }
            }
            GamePhase::Launching(arm) => {
                draw_arm(arm);
                if is_key_pressed(KeyCode::Space) || arm.angle >= arm.max_angle {
                    let klapi = arm.launch_klapi();
                    GamePhase::Launched(
                        arm.clone(),
                        klapi,
                        Kiuas {
                            barriers: vec![
                                Barrier {
                                    bounciness: 0.8,
                                    rect: Rectangle {
                                        position: vec2(3.3, -0.0),
                                        width: 1.0,
                                        height: 0.8,
                                        rotation: 0.0,
                                    },
                                },
                                Barrier {
                                    bounciness: 0.2,
                                    rect: Rectangle {
                                        position: vec2(3.05, -0.9),
                                        width: 0.4,
                                        height: 0.2,
                                        rotation: 0.0,
                                    },
                                },
                            ],
                            goal: Rectangle {
                                position: vec2(3.1, -0.6),
                                width: 0.3,
                                height: 0.3,
                                rotation: 0.0,
                            },
                        },
                        vec![Barrier {
                            bounciness: 0.5,
                            rect: Rectangle {
                                position: vec2(0.0, -1.40),
                                width: 20.0,
                                height: 0.2,
                                rotation: 0.0,
                            },
                        }],
                    )
                } else {
                    GamePhase::Launching(arm.update(time_delta))
                }
            }
            GamePhase::Launched(arm, klapi, kiuas, barriers) => {
                draw_arm(arm);
                if is_key_pressed(KeyCode::R) {
                    GamePhase::Start(new_arm(arm.start_location.clone()))
                } else {
                    draw_sprite_rot(
                        texture_id("klapi"),
                        klapi.rect.position,
                        WHITE,
                        5,
                        klapi.rect.rotation.to_radians(),
                        vec2(klapi.rect.width, klapi.rect.height),
                    );
                    if klapi.rect.collide(&kiuas.goal) {
                        println!("GOAL:{0:?}", kiuas.goal);
                        println!("KLAPI:{0:?}", klapi.rect);
                        self.score = self.score + 10;
                        GamePhase::Start(new_arm(arm.start_location.clone()))
                    } else {
                        let mut updated_klapi = klapi.update(time_delta);
                        let mut kiuas_barriers = kiuas.barriers.clone();
                        let mut all_barriers = barriers.clone();
                        all_barriers.append(&mut kiuas_barriers);
                        for barrier in &all_barriers {
                            if barrier.rect.collide(&updated_klapi.rect) {
                                updated_klapi = barrier.on_collision(&updated_klapi);
                            }
                        }
                        GamePhase::Launched(
                            arm.clone(),
                            updated_klapi,
                            kiuas.clone(),
                            barriers.to_vec(),
                        )
                    }
                }
            }
        };
        egui::Window::new("Score")
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .show(egui(), |ui| {
                ui.label(format!("SCORE: {}", self.score));
            });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_collide_right() {
        let rect1 = Rectangle {
            position: vec2(0.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };
        let rect2 = Rectangle {
            position: vec2(5.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        let result = rect1.collide(&rect2);
        assert_eq!(result, true);
    }

    #[test]
    fn test_collide_left() {
        let rect1 = Rectangle {
            position: vec2(0.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };
        let rect2 = Rectangle {
            position: vec2(-5.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        let result = rect1.collide(&rect2);
        assert_eq!(result, true);
    }

    #[test]
    fn test_collide_top() {
        let rect1 = Rectangle {
            position: vec2(0.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };
        let rect2 = Rectangle {
            position: vec2(0.0, 5.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        let result = rect1.collide(&rect2);
        assert_eq!(result, true);
    }

    #[test]
    fn test_collide_bottom() {
        let rect1 = Rectangle {
            position: vec2(0.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };
        let rect2 = Rectangle {
            position: vec2(-5.0, 0.0),
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        let result = rect1.collide(&rect2);
        assert_eq!(result, true);
    }

    #[test]
    fn test_pivot() {
        let rect = Rectangle {
            position: vec2(0.0, 0.0),
            width: 5.0,
            height: 10.0,
            rotation: 0.0,
        };
        let result = rect.pivot(vec2(0.0, 0.0), 90.0);
        assert_eq!(result.position, rect.position);
        let result3 = rect.pivot(vec2(0.0, 5.0), 45.0);
        assert_eq!(result3.position, vec2(3.535534, 1.464466));
    }

    #[test]
    fn test_pivot_tiny_scale() {
        let rect = Rectangle {
            position: vec2(0.0, 0.0),
            width: 0.18,
            height: 0.80,
            rotation: 0.0,
        };
        let result = rect.pivot(rect.position, 90.0);
        assert_eq!(result.position, rect.position);
        let result3 = rect.pivot(vec2(0.0, 0.40), 45.0);
        assert_eq!(result3.position, vec2(0.282843, 0.117157));
    }

    #[test]
    fn test_bug_in_goal_collision() {
        let goal = Rectangle {
            position: vec2(2.0, -1.3),
            width: 0.3,
            height: 0.2,
            rotation: 0.0,
        };
        let klapi = Rectangle {
            position: vec2(5.933142, -1.0068376),
            width: 0.45,
            height: 0.15,
            rotation: 38.309246,
        };
        assert_eq!(goal.collide(&klapi), false);
        assert_eq!(klapi.collide(&goal), false);
    }
}
