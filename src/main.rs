use comfy::*;

comfy_game!("JMK Klapit", KlapiGame);

pub fn round_to_precision(number: f32, precision: i32) -> f32 {
    let val: f32 = number * 10.0_f32.powi(precision);
    let rounded_val = val.round();
    return rounded_val / 10.0_f32.powi(precision);
}

pub trait GameObject<T> {
    fn update(&self, delta: f32) -> T;
}

pub enum GamePhase {
    Start,
    Charging,
    Launching,
    Launched,
}

pub struct Rectangle {
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

impl Rectangle {
    pub fn collide(&self, rect: &Rectangle) -> bool {
        let tl1 = self.top_left();
        let tl2 = rect.top_left();
        let br1 = self.bottom_right();
        let br2 = rect.bottom_right();
        let y_in_range = tl1.y >= br2.y && br1.y <= tl2.y;
        let x_in_range = tl1.x >= br2.x && br1.x <= tl2.x;
        let collide_right = y_in_range && br1.x >= tl2.x;
        let collide_left = y_in_range && tl1.x >= br2.x;
        let collide_top = x_in_range && tl1.y <= br2.y;
        let collide_bottom = x_in_range && br1.y <= tl2.y;
        return collide_top || collide_bottom || collide_left || collide_right;
    }

    pub fn top_left(&self) -> Vec2 {
        let min_x = self.position.x - self.width / 2.0;
        let max_y = self.position.y + self.height / 2.0;
        return vec2(min_x, max_y);
    }

    pub fn top(&self) -> f32 {
        return self.position.y + self.height / 2.0;
    }

    pub fn max_x(&self) -> f32 {
        let len_c = self.width / 2.0;
        let theta = self.rotation.to_radians();
        let len_b = len_c * theta.cos();
        self.position.x + len_b
    }

    pub fn min_x(&self) -> f32 {
        let len_c = self.width / 2.0;
        let theta = self.rotation.to_radians();
        let len_b = len_c * theta.cos();
        self.position.x - len_b
    }

    pub fn midline_y(&self, x: f32) -> f32 {
        if self.rotation < 90.0 && self.rotation > -90.0{
            let theta = self.rotation.to_radians();
            let delta_x = x - self.position.x;
            let delta_y = theta.tan() * delta_x;
            return delta_y + self.position.y;
        } else {
            return self.position.y;
        }
    }

    pub fn rightline_x(&self, y: f32) -> f32 {
        if self.rotation < 90.0 && self.rotation > -90.0{
            let theta = self.rotation.to_radians();
            let delta_y = y - self.position.y;
            let delta_x = delta_y / theta.tan();
            let alpha = (90.0 - self.rotation).to_radians();
            let width_component = alpha.cos() * self.width * 0.5;
            return delta_x + self.position.x - width_component;
        } else {
            return self.position.x;
        }

    }

    pub fn leftline_x(&self, y: f32) -> f32 {
        if self.rotation < 90.0 && self.rotation > -90.0{
            let theta = self.rotation.to_radians();
            let delta_y = y - self.position.y;
            let delta_x = delta_y / theta.tan();
            let alpha = (90.0 - self.rotation).to_radians();
            let width_component = alpha.cos() * self.width * 0.5;
            return delta_x + self.position.x + width_component;
        } else {
            return self.position.x;
        }

    }

    pub fn topline_y(&self, x: f32) -> f32 {
        let alpha = (90.0 - self.rotation).to_radians();
        let height_component = alpha.sin() * self.height * 0.5;
        let midline = self.midline_y(x);
        return midline + height_component;
    }

    pub fn bottomline_y(&self, x: f32) -> f32 {
        let alpha = (90.0 - self.rotation).to_radians();
        let height_component = alpha.sin() * self.height * 0.5;
        let midline = self.midline_y(x);
        return midline - height_component;
    }

    pub fn bottom(&self) -> f32 {
        return self.position.y - self.height / 2.0;
    }

    pub fn bottom_right(&self) -> Vec2 {
        let max_x = self.position.x + self.width / 2.0;
        let min_y = self.position.y - self.height / 2.0;
        return vec2(max_x, min_y);
    }

    pub fn pivot(&self, pivot_point: Vec2, angle: f32) -> Rectangle {
        if self.contains_point(pivot_point) {
            println!("pivot x:{}, y:{}", pivot_point.x, pivot_point.y);
            let relative_x = pivot_point.x - self.position.x;
            let relative_y = pivot_point.y - self.position.y;
            let dist = (relative_x.powf(2.0) + relative_y.powf(2.0)).sqrt();
            println!("distance {}", dist);
            let theta = -1.0 * angle.to_radians();
            let rot_corner_x = pivot_point.x - theta.sin() * dist;
            let rot_corner_y = pivot_point.y - theta.cos() * dist;
            let position = vec2(
                round_to_precision(rot_corner_x, 6),
                round_to_precision(rot_corner_y, 6),
            );
            println!("rot_corner x:{}, y:{}", position.x, position.y);
            return Rectangle {
                position,
                width: self.width,
                height: self.height,
                rotation: angle,
            };
        } else {
            self.clone()
        }
    }

    fn contains_point(&self, point: Vec2) -> bool {
        let tl = self.top_left();
        let br = self.bottom_right();
        return tl.x <= point.x && br.x >= point.x && tl.y >= point.x && br.y <= point.y;
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

pub struct Klapi {
    pub rect: Rectangle,
    pub mass: f32,
    pub forces: Vec<Vec2>,
    pub speed: Vec2,
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
        let x_change = x_force_sum * delta;
        let y_change = y_force_sum * delta;
        let new_x_speed = self.speed.x + 0.17 * x_change;
        let new_y_speed = self.speed.y + 0.17 * y_change;
        return vec2(new_x_speed, new_y_speed);
    }

    fn calculate_new_rect(&self, speed: Vec2, delta: f32) -> Rectangle {
        let old_rect = &self.rect;

        let x = self.rect.position.x + speed.x * delta;
        let y = self.rect.position.y + speed.y * delta;
        let new_position = vec2(x, y);
        return Rectangle {
            height: old_rect.height,
            width: old_rect.width,
            rotation: old_rect.rotation,
            position: new_position,
        };
    }
}

pub fn new_klapi() -> Klapi {
    Klapi {
        rect: Rectangle {
            position: vec2(0.0, 30.0),
            height: 0.15,
            width: 0.45,
            rotation: 0.0,
        },
        speed: vec2(0.0, 0.0),
        max_speed: 10.0,
        mass: 2.5,
        forces: vec![vec2(0.0, -9.81 * 2.5)],
    }
}

impl GameObject<Klapi> for Klapi {
    fn update(&self, delta: f32) -> Klapi {
        let speed = self.calculate_new_speed(delta);
        let rect = self.calculate_new_rect(speed, delta);
        return Klapi {
            rect,
            speed,
            forces: self.forces.clone(),
            max_speed: self.max_speed,
            mass: self.mass,
        };
    }
}

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
        let arm_rect = self.get_arm_start_rect().pivot(self.pivot_location, angle);

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
    let height = 0.7;
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
            position: pivot_location,
            rotation: 0.0,
        },
        angle,
        max_angle: 90.0,
        min_angle: -90.0,
        speed: 3.0,
        thrown: false,
        radius: height / 2.0,
        hand_rect: Rectangle {
            position: hand_position,
            height: width,
            width,
            rotation: angle,
        },
    };
}

pub struct Barrier {
    pub bounciness: f32,
    pub rect: Rectangle,
}

impl Barrier {
    fn on_collision(&self, klapi: &Klapi) -> Klapi {
        println!("COLLISION");
        let rect = self.move_klapi_out_of(klapi);
        let velo = (klapi.speed.x.powf(2.0) + klapi.speed.x.powf(2.0)).sqrt();
        let x_speed = self.bounciness * velo * (0.0_f32).cos();
        let y_speed = self.bounciness * velo * (0.0_f32).sin();
        let speed = vec2(x_speed, y_speed);
        let mut forces = if klapi.rect.bottom() > self.rect.top() {
            vec![vec2(0.0, 9.81)]
        } else {
            vec![]
        };
        for f in &klapi.forces {
            forces.push(*f);
        }
        return Klapi {
            forces,
            speed,
            rect,
            mass: klapi.mass,
            max_speed: klapi.max_speed,
        };
    }

    fn move_klapi_out_of(&self, klapi: &Klapi) -> Rectangle {
        let resolution = 0.1;
        let mut rect = klapi.rect.clone();
        while self.rect.collide(&rect) {
            let old_pos = klapi.rect.position;
            let x = old_pos.x - klapi.speed.x * resolution;
            let y = old_pos.y - klapi.speed.y * resolution;
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

pub struct Kiuas {
    pub barriers: Vec<Barrier>,
    pub goal: Rectangle,
}

pub struct KlapiGame {
    pub phase: GamePhase,
    pub arm: Arm,
    pub klapi: Klapi,
    pub barriers: Vec<Barrier>,
}

impl GameLoop for KlapiGame {
    fn new(_c: &mut EngineState) -> Self {
        let mut camera = main_camera_mut();
        camera.zoom = 2.0;
        Self {
            phase: GamePhase::Start,
            klapi: new_klapi(),
            arm: new_arm(vec2(0.0, 0.0)),
            barriers: vec![Barrier {
                bounciness: 0.6,
                rect: Rectangle {
                    width: 100.0,
                    height: 20.0,
                    rotation: 0.0,
                    position: vec2(-15.0, -20.0),
                },
            }],
        }
    }

    fn update(&mut self, _c: &mut EngineContext) {
        let time_delta = delta();
        draw_rect(
            self.klapi.rect.position,
            vec2(self.klapi.rect.width, self.klapi.rect.height),
            RED.alpha(0.8),
            5,
        );
        self.arm = self.arm.update(time_delta);
        println!(
            "Arm tl {}, arm br {}",
            self.arm.arm_rect.top_left(),
            self.arm.arm_rect.bottom_right()
        );
        self.klapi = self.klapi.update(time_delta);
        println!(
            "Arm rect x: {}, y:{}, angle: {}",
            self.arm.arm_rect.position.x, self.arm.arm_rect.position.y, self.arm.arm_rect.rotation
        );
        draw_circle(self.arm.pivot_location, 0.01, BLACK.alpha(1.0), 6);
        draw_circle(self.arm.arm_rect.position, 0.01, BLACK.alpha(1.0), 6);
        draw_circle(
            self.arm.pivot_location,
            self.arm.radius,
            GREEN.alpha(0.3),
            5,
        );
        draw_rect(
            self.arm.arm_rect.position,
            vec2(self.arm.arm_rect.width, self.arm.arm_rect.height),
            BLUE.alpha(0.8),
            5,
        );

        draw_line(
            vec2(-1.0, self.arm.arm_rect.midline_y(-1.0)),
            vec2(1.0, self.arm.arm_rect.midline_y(1.0)),
            0.1,
            PINK.alpha(0.8),
            5,
        );
        draw_line(
            vec2(self.arm.arm_rect.rightline_x(-1.0), -1.0),
            vec2(self.arm.arm_rect.rightline_x(1.0), 1.0),
            0.1,
            PINK.alpha(0.8),
            5,
        );
        draw_rect_rot(
            self.arm.arm_rect.position,
            vec2(self.arm.arm_rect.width, self.arm.arm_rect.height),
            self.arm.angle.to_radians(),
            YELLOW.alpha(0.8),
            5,
        );
        for barrier in &self.barriers {
            draw_rect(
                barrier.rect.position,
                vec2(barrier.rect.width, barrier.rect.height),
                BLUE.alpha(0.8),
                5,
            );
            if barrier.rect.collide(&self.klapi.rect) {
                self.klapi = barrier.on_collision(&self.klapi);
            }
        }
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
    fn test_contains_point() {
        let rect = Rectangle {
            position: vec2(0.0, 0.0),
            width: 5.0,
            height: 10.0,
            rotation: 0.0,
        };
        let result = rect.contains_point(vec2(0.0, 5.0));
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
    fn test_midline_y_rot_45() {
        let rect = Rectangle {
            position: vec2(0.0, 3.0),
            width: 4.0,
            height: 2.0,
            rotation: 45.0,
        };
        for i in 1..10 {
            let f = i as f32;
            let result = round_to_precision(rect.midline_y(f), 4);
            let expected = round_to_precision(f + 3.0, 4);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_midline_y_rot_89() {
        let rect = Rectangle {
            position: vec2(0.0, 0.0),
            width: 4.0,
            height: 2.0,
            rotation: 89.0,
        };
        let result = round_to_precision(rect.midline_y(1.0), 4);
        let expected = round_to_precision(57.29, 4);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_topline_y_rot_0() {
        let rect = Rectangle {
            position: vec2(0.0, 0.0),
            width: 4.0,
            height: 2.0,
            rotation: 0.0,
        };

        let mut result = rect.topline_y(0.0);
        assert_eq!(result, 1.0);
        result = rect.topline_y(1.0);
        assert_eq!(result, 1.0);
        result = rect.topline_y(-1.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_topline_y_rot_45() {
        let rect = Rectangle {
            position: vec2(0.0, 0.0),
            width: 4.0,
            height: 2.0,
            rotation: 45.0,
        };
        for i in 1..10 {
            let f = i as f32;
            let result = round_to_precision(rect.topline_y(f), 4);
            let expected = round_to_precision(f + (45.0_f32).to_radians().cos() * 1.0, 4);
            assert_eq!(result, expected);
        }
    }
}
