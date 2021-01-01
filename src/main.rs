// 1. find random random position at the bottom of the screen
// 2. Set a random distance to travel
// 3. Animate an explosion

use std::f32::consts::{PI, TAU};

use rand::prelude::*;
use tinybit::events::{events, Event, EventModel, KeyCode, KeyEvent};
use tinybit::{term_size, Color, Pixel, Renderer, ScreenPos, ScreenSize, StdoutTarget, Viewport};

fn random_color(rng: &mut ThreadRng) -> Color {
    Color::Rgb {
        r: rng.gen_range(0..255),
        g: rng.gen_range(0..255),
        b: rng.gen_range(0..255),
    }
}

// -----------------------------------------------------------------------------
//     - Explosion -
// -----------------------------------------------------------------------------
const MAX_R: f32 = 10.0;

fn rando_float(rng: &mut ThreadRng) -> f32 {
    (rng.gen_range(0..100) as f32 / 100.0)
}

struct Explosion {
    origin: ScreenPos,
    pixels: Vec<(Pixel, ScreenPos)>, // ScreenPos = TargetPos
    speed: u64,
    speed_target: u64,
    life: u8,
}

impl Explosion {
    fn new(origin: ScreenPos, rng: &mut ThreadRng) -> Self {
        let num_particles = rng.gen_range(3..10);

        let pixels = (0..num_particles)
            .map(|i| {
                let p = Pixel::new('*', origin, Some(random_color(rng)), None);

                let angle = TAU * rando_float(rng);
                let r = MAX_R * rando_float(rng).sqrt();
                let x = r * angle.cos() + origin.x as f32;
                let y = r * angle.sin() + origin.y as f32;

                let target = ScreenPos::new(x as u16, y as u16);

                (p, target)
            })
            .collect();

        Self {
            origin,
            pixels,
            speed: 1,
            speed_target: 0,
            life: rng.gen_range(2..7),
        }
    }

    fn pixels(&self) -> Vec<Pixel> {
        self.pixels.iter().map(|(p, _)| *p).collect()
    }

    fn fly(&mut self, fps: u64, rng: &mut ThreadRng) {
        if self.life == 0 {
            return;
        }

        self.speed_target += fps;
        if self.speed <= self.speed_target {
            self.speed_target = 0;
            let speed = self.speed as f32;
            self.life -= 1;

            if self.life == 0 {
                self.pixels.clear();
            }

            self.pixels.iter_mut().for_each(|(pix, target)| {
                let mut pos = pix.pos.cast::<f32>();
                let target = target.cast::<f32>();
                let dir = (pos.to_vector() - target.to_vector()).normalize();
                pos += (dir * speed);

                pos.x = pos.x.max(0.0);
                pos.y = pos.y.max(0.0);

                // EO horrid things
                match pos.try_cast::<u16>() {
                    Some(v) => pix.pos = v,
                    None => panic!(pos),
                }
            });
        }
    }
}

// -----------------------------------------------------------------------------
//     - Firework -
// -----------------------------------------------------------------------------
struct Firework {
    pos: ScreenPos,
    target: ScreenPos,
    lifetime_ms: i64,
    speed: u64,
    speed_target: u64,
    explosion: Option<Explosion>,
    color: Color,
}

impl Firework {
    fn pixels(&self) -> Vec<Pixel> {
        match self.explosion {
            Some(ref e) => e.pixels(),
            None => vec![Pixel::new('#', self.pos, Some(self.color), None)],
        }
    }

    fn fly(&mut self, fps: u64, rng: &mut ThreadRng) {
        self.speed_target += fps;

        if self.speed <= self.speed_target {
            self.speed_target = 0;

            if self.target.y < self.pos.y {
                self.pos.y -= 1;
            } else {
                match self.explosion {
                    None => self.explosion = Some(Explosion::new(self.target, rng)),
                    Some(ref mut e) => e.fly(fps, rng),
                }
            }
        }

    }
}

fn spawn_firework(rng: &mut ThreadRng) -> Firework {
    let (w, h) = term_size().unwrap();
    let x = rng.gen_range(0..w);
    let target_y = rng.gen_range(8..h - 3);

    Firework {
        pos: ScreenPos::new(x, h - 1),
        target: ScreenPos::new(x, target_y),
        lifetime_ms: rng.gen_range(1000..2000),
        speed: rng.gen_range(1..10),
        speed_target: 0,
        explosion: None,
        color: random_color(rng),
    }
}

// -----------------------------------------------------------------------------
//     - Main -
// -----------------------------------------------------------------------------
fn main() {
    let mut rng = thread_rng();

    let mut fireworks = vec![];
    let mut dead_fireworks = vec![];

    let max_fireworks = 10;
    let fps = 20;

    let (w, h) = term_size().unwrap();
    let mut viewport = Viewport::new(ScreenPos::new(0, 0), ScreenSize::new(w, h));
    let stdout_rend = StdoutTarget::new().unwrap();
    let mut renderer = Renderer::new(stdout_rend);

    for event in events(EventModel::Fps(fps)) {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,
            Event::Tick => {
                if fireworks.len() < max_fireworks {
                    let firework = spawn_firework(&mut rng);
                    fireworks.push(firework);
                }

                fireworks.iter_mut().enumerate().for_each(|(i, f)| {
                    f.lifetime_ms -= fps as i64;
                    if f.lifetime_ms <= 0 {
                        dead_fireworks.push(i);
                    }

                    f.fly(fps, &mut rng);
                    viewport.draw_pixels(f.pixels());
                });

                renderer.render(&mut viewport);

                dead_fireworks.sort();
                while let Some(i) = dead_fireworks.pop() {
                    fireworks.remove(i);
                }
            }
            _ => {}
        }
    }
}
