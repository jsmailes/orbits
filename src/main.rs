extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate fps_counter;

use std::collections::VecDeque;

use glutin_window::GlutinWindow as Window;
use window::AdvancedWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use rand::Rng;
use rand::prelude::ThreadRng;

use fps_counter::FPSCounter;


struct Planet {
    color: [f32; 4],
    mass: f64,
    radius: f64,
    x: f64,
    y: f64,
}

struct Satellite {
    color: [f32; 4],
    radius: f64,
    x: f64,
    y: f64,
    v_x: f64,
    v_y: f64,
    trail: VecDeque<(f64, f64)>,
}

struct Args {
    title: String,         // Window title
    width: f64,            // Viewport width
    height: f64,           // Viewport height
    add_chance: f64,       // Chance to add a satellite each frame
    sat_radius: f64,       // Radius (in px) of each satellite
    sat_velocity: f64,     // Initial velocity (in px/s) of each satellite
    gravity_constant: f64, // 'G' constant used to update velocities
    trail_length: usize,     // Trail length, measured in number of frames of history
}

// Returns true if the point with given radius is outside the window, for given window size
fn outside(x: f64, y: f64, radius: f64, width: f64, height: f64) -> bool {
    (x + radius < 0.0)
    | (y + radius < 0.0)
    | (x - radius > width)
    | (y + radius > height)
}

// Returns a random color
fn random_color(rng: &mut ThreadRng) -> [f32; 4] {
    [rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), 1.0]
}


pub struct App {
    gl: GlGraphics,              // OpenGL drawing backend
    rng: ThreadRng,              // Random number generator
    fps_counter: FPSCounter,     // FPS counter
    planets: Vec<Planet>,        // Data for planets
    satellites: Vec<Satellite>,  // Data for satellites
    args: Args,                  // Any other useful arguments
}

impl App {
    fn render(&mut self, args: &RenderArgs, window: &mut Window) {
        let fps = self.fps_counter.tick();
        window.set_title(format!("{} ({} fps)", self.args.title, fps));

        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let planets_iter = self.planets.iter();
        let satellites_iter = self.satellites.iter();

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            // Draw planets
            for planet in planets_iter {
                let rect = rectangle::rectangle_by_corners(planet.x - planet.radius, planet.y - planet.radius, planet.x + planet.radius, planet.y + planet.radius);
                ellipse(planet.color, rect, c.transform, gl);
            }

            // Draw satellites
            for satellite in satellites_iter {
                // Draw trail
                if satellite.trail.len() > 1 {
                    let mut pos_old = satellite.trail[0];
                    for pos in satellite.trail.iter().skip(1) {
                        line(satellite.color, 1.0, [pos.0, pos.1, pos_old.0, pos_old.1], c.transform, gl);
                        pos_old = *pos;
                    }
                }

                let rect = rectangle::rectangle_by_corners(satellite.x - satellite.radius, satellite.y - satellite.radius, satellite.x + satellite.radius, satellite.y + satellite.radius);
                ellipse(satellite.color, rect, c.transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let width = self.args.width;
        let height = self.args.height;

        // Chance to add a new satellite
        // TODO make dependent on args.dt
        let c: f64 = self.rng.gen_range(0.0..1.0);
        if c < self.args.add_chance {
            // Add new satellite
            let color: [f32; 4] = random_color(&mut self.rng);
            let x: f64 = self.rng.gen_range(0.0..width);
            let y: f64 = self.rng.gen_range(0.0..height);
            let angle: f64 = self.rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            let v_x: f64 = self.args.sat_velocity * angle.cos();
            let v_y: f64 = self.args.sat_velocity * angle.sin();
            let sat = Satellite {
                color,
                radius: self.args.sat_radius,
                x,
                y,
                v_x,
                v_y,
                trail: VecDeque::new(),
            };
            self.satellites.push(sat);
        }


        // Update satellites
        for sat in self.satellites.iter_mut() {
            // Update velocities
            for planet in self.planets.iter() {
                let distance_x = sat.x - planet.x;
                let distance_y = sat.y - planet.y;
                let distance_sq = (distance_x * distance_x) + (distance_y * distance_y);
                let delta_velocity = (self.args.gravity_constant * planet.mass * args.dt) / (distance_sq);
                let angle = distance_y.atan2(distance_x);
                sat.v_x += -1.0 * delta_velocity * angle.cos();
                sat.v_y += -1.0 * delta_velocity * angle.sin();
            }

            // Update positions
            sat.x += sat.v_x * args.dt;
            sat.y += sat.v_y * args.dt;

            // Update trails
            sat.trail.push_back((sat.x, sat.y));
            while sat.trail.len() > self.args.trail_length {
                sat.trail.pop_front();
            }
        }

        // Destroy satellites if they pass outside the screen or hit a planet
        let planets = &(self.planets);
        self.satellites.retain(|sat| {
            !(
                (outside(sat.x, sat.y, sat.radius, width, height)
                & sat.trail.iter().all(|pos| outside(pos.0, pos.1, 1.0, width, height)))
                | planets.iter().any(|planet| {
                    let distance_x = sat.x - planet.x;
                    let distance_y = sat.y - planet.y;
                    let distance_sq = (distance_x * distance_x) + (distance_y * distance_y);
                    distance_sq.sqrt() < sat.radius + planet.radius
                })
            )
            // TODO can clean this up by adding a "dead" flag
        });
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let width = 800;
    let height = 800;

    let mut rng = rand::thread_rng();

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("orbits", [width, height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create planets/satellites
    let mut planets: Vec<Planet> = Vec::new();
    planets.push(Planet {
        color: random_color(&mut rng), //[0.0, 1.0, 1.0, 1.0],
        mass: 1000.0,
        radius: 25.0,
        x: width as f64 / 2.0 - 200.0,
        y: height as f64 / 2.0,
    });
    planets.push(Planet {
        color: random_color(&mut rng), //[0.0, 1.0, 1.0, 1.0],
        mass: 1000.0,
        radius: 25.0,
        x: width as f64 / 2.0 + 200.0,
        y: height as f64 / 2.0,
    });

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rng,
        fps_counter: FPSCounter::default(),
        planets: planets,
        satellites: Vec::new(),
        args: Args {
            title: "orbits".to_string(),
            width: width as f64,
            height: height as f64,
            add_chance: 0.01,
            sat_radius: 5.0,
            sat_velocity: 200.0,
            gravity_constant: 4000.0,
            trail_length: 100,
        }
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &mut window);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
