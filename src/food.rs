use nannou::glam::Vec2;
use rand::Rng;

#[derive(PartialEq, Clone, Copy)]
pub struct Food {
    pub position: Vec2,
    pub hue: f32,
    pub radius: f32,

    pub saturation: f32,
}

impl Food {
    pub fn new(position: Vec2, saturation: f32, rng: &mut impl Rng) -> Self {
        
        let hue = if saturation < 0. {
            rng.gen_range(0.0..0.10)
        } else {
            rng.gen_range(0.25..0.40)
        };
        
        Food {
            position,
            saturation,
            radius: saturation.abs()*0.5,
            hue
        }
    }

    pub fn new_food(position: Vec2, rng: &mut impl Rng) -> Self {
        let saturation = rng.gen_range(15.0..25.0);

        Food::new(position, saturation, rng)
    }

    pub fn new_poison(position: Vec2, rng: &mut impl Rng) -> Self {
        let saturation = -rng.gen_range(15.0..25.0);

        Food::new(position, saturation, rng)
    }
}
