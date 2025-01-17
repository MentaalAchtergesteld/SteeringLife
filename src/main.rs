use std::f32::INFINITY;

use food::Food;
use nannou::{color, event::{Key, Update}, glam::{vec2, Vec2}, prelude::Pow, App, Draw, Frame};
use nannou_egui::{egui, Egui};
use rand::{rngs::ThreadRng, Rng};
use steering_agent::{Dna, SteeringAgent};

mod steering_agent;
mod food;


struct Model {
    rng: ThreadRng,
    egui: Egui,

    agents: Vec<SteeringAgent>,
    food: Vec<Food>,
    poison: Vec<Food>,
    
    minimum_food_count: usize,
    minimum_poison_count: usize,
    minimum_agent_count: usize,

    debug: bool,
    follow_mouse: bool
}

fn main() {
    nannou::app(init)
        .update(update)
        .run();
}

fn init(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("Steering Life")
        .key_pressed(key_pressed)
        .raw_event(raw_window_event)
        .view(draw)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    let mut rng = rand::thread_rng();

    let window_width = app.window_rect().w();
    let window_height = app.window_rect().h();

    let agent_count = 4;

    let agents = (0..agent_count).map(|_| {
        SteeringAgent::new(
            vec2(
                rng.gen_range(-window_width/2.0..window_width/2.0),
                rng.gen_range(-window_height/2.0..window_height/2.0),
            ),
            &Dna::random(&mut rng),
        )
    }).collect::<Vec<SteeringAgent>>();

    let food_count = 64;

    let food = (0..food_count).map(|_| {
        let position = vec2(
            rng.gen_range(-window_width/2.0..window_width/2.0),
            rng.gen_range(-window_height/2.0..window_height/2.0),
        );

        Food::new_food(position, &mut rng)
    }).collect::<Vec<Food>>();

    let poison_count = 32;

    let poison = (0..poison_count).map(|_| {
        let position = vec2(
            rng.gen_range(-window_width/2.0..window_width/2.0),
            rng.gen_range(-window_height/2.0..window_height/2.0),
        );

        Food::new_poison(position, &mut rng)
    }).collect::<Vec<Food>>();

    let egui = Egui::from_window(&window);

    Model {
        agents,
        food,
        poison,
        rng,
        egui,

        minimum_food_count: food_count,
        minimum_poison_count: poison_count,
        minimum_agent_count: agent_count,
        debug: false,
        follow_mouse: false,
    }
}

fn find_closest_food(position: Vec2, max_distance: f32, food: &Vec<Food>) -> Option<(usize, Food)> {
    let mut closest = None;
    let mut closest_distance = INFINITY;

    let max_distance_squared = max_distance.pow(2);

    for (index, food) in food.iter().enumerate() {
        let distance = position.distance_squared(food.position);

        if distance < max_distance_squared && distance < closest_distance {
            closest = Some((index, *food));
            closest_distance = distance;
        }
    }

    closest
}

fn update(app: &App, model: &mut Model, update: Update) {
    let delta = update.since_last.as_secs_f32();

    let max_hunger_before_dead = 128.0;
    let max_hunger_before_search = 20.0;

    let mut newborns = Vec::new();

    let mut average_age = 0.;
    let mut agent_count = 0;

    model.agents.retain_mut(|agent| {
        if model.follow_mouse {
            agent.arrive(app.mouse.position(), 1.0);
            agent.hunger = 0.;
            agent.update();
            return true;
        }

        agent.age += delta;

        average_age += agent.age;
        agent_count += 1;

        let mut should_retain = true;

        let mut touched_poison = false;
        model.poison.retain(|poison| {
            let distance_squared = agent.position.distance_squared(poison.position);

            if distance_squared < poison.radius.pow(2) {
                touched_poison = true;
                return false;
            } else {
                if distance_squared < agent.dna.poison_detection_radius.pow(2) {
                    agent.flee(poison.position, agent.dna.poison_force_multiplier);
                }
                true
            }
        });

        if touched_poison {
            should_retain = false;
        } else if agent.hunger > max_hunger_before_dead {
            should_retain = false;
        } else if agent.hunger > max_hunger_before_search {
            let closest_food = find_closest_food(agent.position, agent.dna.food_detection_radius, &model.food);

            if let Some((index, food)) = closest_food {
                agent.arrive(food.position, agent.dna.food_force_multiplier);

                if agent.position.distance_squared(food.position) < food.radius.pow(2) {
                    model.food.remove(index);
                    agent.hunger -= food.saturation;
                }
            } else {
                agent.wander(app.window_rect(), &mut model.rng);
            }
        } else {
            agent.wander(app.window_rect(), &mut model.rng);
        }

        agent.update();

        let should_create_child = model.rng.gen_bool(0.001);

        if should_create_child {
            let new_agent = SteeringAgent::new(agent.position, &agent.dna.mutate(0.75, &mut model.rng));
            newborns.push(new_agent);
        }

        should_retain
    });

    average_age /= agent_count as f32;

    model.agents.append(&mut newborns);

    let window_width = app.window_rect().w();
    let window_height = app.window_rect().h();

    if model.food.len() < model.minimum_food_count {
        let difference = model.minimum_food_count - model.food.len();

        for _ in 0..difference {
            if model.rng.gen_bool(0.3) {
                model.food.push(Food::new_food(
                    vec2(model.rng.gen_range(-window_width/2.0..window_width/2.0), model.rng.gen_range(-window_height/2.0..window_height/2.0)),
                        &mut model.rng
                    )
                );
            }
        }
    }

    if model.poison.len() < model.minimum_poison_count {
        let difference = model.minimum_poison_count - model.poison.len();

        for _ in 0..difference {
            model.poison.push(Food::new_poison(
                vec2(model.rng.gen_range(-window_width/2.0..window_width/2.0), model.rng.gen_range(-window_height/2.0..window_height/2.0)),
                    &mut model.rng
                )
            );
        }
    }

    if model.agents.len() < model.minimum_agent_count {
        let difference = model.minimum_agent_count - model.agents.len();

        for _ in 0..difference {
            model.agents.push(SteeringAgent::new(
                vec2(model.rng.gen_range(-window_width/2.0..window_width/2.0), model.rng.gen_range(-window_height/2.0..window_height/2.0)),
                    &Dna::random(&mut model.rng)
                )
            );
        }
    }

    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Steering Life").show(&ctx, |ui| {
        ui.label(format!("Average lifespan: {:.2}", average_age));

        ui.label(format!("{:.2} FPS", app.fps()));
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::D => model.debug = !model.debug,
        Key::F => model.follow_mouse = !model.follow_mouse,
        _ => {},
    }
}

fn draw_agents(model: &Model, draw: &Draw) {
    let agent_width = 24.;
    let agent_height = 16.;

    let triangle_top = vec2(agent_width/2., 0.);
    let triangle_left = vec2(-agent_width/3., -agent_height/2.);
    let triangle_right = vec2(-agent_width/3., agent_height/2.);

    for agent in &model.agents {
        draw.tri()
            .color(color::hsv(agent.dna.hue, 0.7, 0.7))
            .points(
                triangle_top,
                triangle_left,
                triangle_right
            )
            .rotate(agent.direction)
            .xy(agent.position);

        if model.debug {
            // Food force multiplier
            draw.line()
                .end(Vec2::X * agent.dna.food_force_multiplier * 10.)
                .weight(4.0)
                .color(color::GREEN)
                .caps_round()
                .rotate(agent.direction)
                .xy(agent.position);
    
            // Poison force multiplier
            draw.line()
                .end(-Vec2::X * agent.dna.poison_force_multiplier * 10.)
                .weight(4.0)
                .color(color::RED)
                .caps_round()
                .rotate(agent.direction)
                .xy(agent.position);
    
            // Food detection radius
            draw.ellipse()
                .xy(agent.position)
                .radius(agent.dna.food_detection_radius)
                .stroke_weight(4.0)
                .stroke(color::GREEN)
                .no_fill();
    
            // Poison detection radius
            draw.ellipse()
                .xy(agent.position)
                .radius(agent.dna.poison_detection_radius)
                .stroke_weight(4.0)
                .stroke(color::RED)
                .no_fill();
        }
    }
}

fn draw_food_and_poison(model: &Model, draw: &Draw) {
    for food in &model.food {
        draw.ellipse()
            .color(color::hsv(food.hue, 0.7, 0.7))
            .radius(food.radius)
            .xy(food.position);
    }

    for poison in &model.poison {
        draw.ellipse()
            .color(color::hsv(poison.hue, 0.7, 0.7))
            .radius(poison.radius)
            .xy(poison.position);
    }
}

fn draw(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(color::gray(0.1));

    draw_agents(model, &draw);    

    draw_food_and_poison(model, &draw);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}