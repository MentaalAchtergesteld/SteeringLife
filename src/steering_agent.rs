use nannou::{geom::Rect, glam::{vec2, Vec2}, math::Vec2Angle};
use rand::Rng;

fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
}

fn get_random_position(rect: Rect, rng: &mut impl Rng) -> Vec2 {
    vec2(
        rng.gen_range(-rect.w()/2.0..rect.w()/2.0),
        rng.gen_range(-rect.h()/2.0..rect.h()/2.0)
    )
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Dna {
    pub hue: f32,
    pub max_velocity: f32,
    pub max_steer_force: f32,
    pub arrive_threshold: f32,
    pub food_detection_radius: f32,
    pub food_force_multiplier: f32,
    pub poison_detection_radius: f32,
    pub poison_force_multiplier: f32
}

impl Into<[f32; 8]> for Dna {
    fn into(self) -> [f32; 8] {
        [
            self.max_velocity,
            self.max_steer_force,
            self.arrive_threshold,
            self.food_detection_radius,
            self.food_force_multiplier,
            self.poison_detection_radius,
            self.poison_force_multiplier,
            self.hue
        ]
    }
}

impl From<[f32; 8]> for Dna {
    fn from(value: [f32; 8]) -> Self {
        Dna {
            max_velocity: value[0],
            max_steer_force: value[1],
            arrive_threshold: value[2],
            food_detection_radius: value[3],
            food_force_multiplier: value[4],
            poison_detection_radius: value[5],
            poison_force_multiplier: value[6],
            hue: value[7],
        }
    }
}

impl Dna {
    pub fn mutate(self, mutate_chance: f64, rng: &mut impl Rng) -> Dna {
        let mut dna_array: [f32; 8] = self.into();

        for i in 0..dna_array.len() {
            if rng.gen_bool(mutate_chance) {
                dna_array[i] *= rng.gen_range(0.93..1.07);
            }
        };

        let mutated_dna = Dna::from(dna_array);

        mutated_dna
    }

    pub fn random(rng: &mut impl Rng) -> Dna {
        Dna {
            hue: rng.gen_range(0.0..1.0),
            max_velocity: rng.gen_range(2.5..5.0),
            max_steer_force: rng.gen_range(0.075..0.2),
            arrive_threshold: rng.gen_range(48.0..64.0),
            food_detection_radius: rng.gen_range(64.0..192.0),
            food_force_multiplier: rng.gen_range(0.75..1.25),
            poison_detection_radius: rng.gen_range(64.0..192.0),
            poison_force_multiplier: rng.gen_range(0.75..1.25),
        }
    }
}

#[derive(Default)]
pub struct SteeringAgent {
    pub position: Vec2,
    pub velocity: Vec2,
    acceleration: Vec2,
    
    pub direction: f32,

    pub hunger: f32,

    pub wander_target: Option<Vec2>,

    pub dna: Dna,
}

impl SteeringAgent {
    pub fn new(position: Vec2, dna: &Dna) -> Self {
        SteeringAgent {
            position,
            dna: dna.clone(),
            ..Default::default()
        }
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.acceleration += force;
    }

    fn update_position(&mut self) {
        let min_velocity_threshold = 0.01;

        self.velocity += self.acceleration;
        if self.velocity.length() < min_velocity_threshold {
            self.velocity.x = 0.;
            self.velocity.y = 0.;
        }

        if self.velocity.length() > 0. {
            self.direction = self.velocity.angle();
        }

        self.velocity = self.velocity.clamp_length_max(self.dna.max_velocity);

        self.position += self.velocity;

        self.acceleration.x = 0.;
        self.acceleration.y = 0.;
    }

    fn update_hunger(&mut self) {
        self.hunger += 0.2;
    }

    pub fn update(&mut self) {
        self.update_position();
        self.update_hunger();
    }

    pub fn seek(&mut self, target: Vec2, force_multiplier: f32) {
        let desired_velocity = (target - self.position).normalize() * self.dna.max_velocity;
        let steering_force = (desired_velocity - self.velocity).clamp_length_max(self.dna.max_steer_force);
        self.apply_force(steering_force * force_multiplier);
    }

    pub fn arrive(&mut self, target: Vec2, force_multiplier: f32) {
        let position_difference = target - self.position;
    
        let distance = position_difference.length();
        
        let desired_velocity = if distance < self.dna.arrive_threshold {
            let arrive_speed = map_range(distance, 0., self.dna.arrive_threshold, 0., self.dna.max_velocity);
            position_difference.normalize() * arrive_speed
        } else {
            position_difference.normalize() * self.dna.max_velocity
        };
    
        let steering_force = (desired_velocity - self.velocity).clamp_length_max(self.dna.max_steer_force);
        self.apply_force(steering_force * force_multiplier);
    }

    pub fn flee(&mut self, target: Vec2, force_multiplier: f32) {
        let desired_velocity = (target - self.position).normalize() * self.dna.max_velocity;
        let steering_force = (desired_velocity - self.velocity).clamp_length_max(self.dna.max_steer_force);
        self.apply_force(steering_force*-1. * force_multiplier);
    }
    
    pub fn wander(&mut self, wander_rect: Rect, rng: &mut impl Rng) {
        let wander_target = if let Some(wander_target) = self.wander_target {
            wander_target
        } else {
            let wander_target = get_random_position(wander_rect, rng);
            self.wander_target = Some(wander_target);
            wander_target
        };
    
        self.arrive(wander_target, 1.0);
    
        if self.position.distance_squared(wander_target) < 16. {
            self.wander_target = None;
        }
    }
}