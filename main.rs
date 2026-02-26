mod geometry;

use core::f64;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Duration;

use sdl3::event::Event;
use sdl3::pixels::Color;
use sdl3::render::WindowCanvas;

use crate::geometry::{Object, Ray, Sphere, Vector};

fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("cornea", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut scene = get_scene();

    let num_frames = 60;
    let mut counter = 0;
    'running: loop {
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255u8, 255u8, 255u8));

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
        //     // The rest of the game loop goes here...

        for lid in 3..=4 {
            scene.get_light_source(lid).and_modify(|light| {
                let x = f64::sin(
                    ((counter % num_frames) as f64 / num_frames as f64) * 0.1 - 0.05
                        + std::f64::consts::PI,
                );
                let z = f64::cos(
                    ((counter % num_frames) as f64 / num_frames as f64) * 0.1 - 0.05
                        + std::f64::consts::PI,
                );

                light.x = 25000. * x;
                light.z = 25000. * z;
            });
        }

        capture(&scene, &mut canvas);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        counter += 1;
    }
}

fn get_scene() -> Scene {
    let mut scene = Scene::new();

    scene.add_object(
        1,
        Box::new(Sphere::new(Vector::new(0., 0., 10000.), 125., 1)),
    );

    scene.add_object(
        2,
        Box::new(Sphere::new(Vector::new(0., 100., 5000.), 125., 2)),
    );

    // scene.add_light_source(3, Vector::new(25000., 25000., 25000.));
    scene.add_light_source(4, Vector::new(0., 200., 0.));

    scene
}

fn capture(scene: &Scene, canvas: &mut WindowCanvas) {
    let (width, height) = canvas.output_size().unwrap();

    for y in 0..width {
        for x in 0..height {
            let norm_x = x as f64 - (width as f64 / 2.);
            let norm_y = (height as f64 / 2.) - y as f64;

            let point = Vector::new(norm_x, norm_y, 0.);
            let end_point = Vector::new(norm_x, norm_y, 1.0);
            let cam_ray = Ray::new(point.clone(), end_point.clone());

            let capped_light = scene.get_reflected_light(&cam_ray) as u8;

            canvas.set_draw_color(Color::RGB(capped_light, capped_light, capped_light));
            canvas.draw_point((x as f32, y as f32)).unwrap();
        }
    }
}

pub struct Scene {
    object_lookup: HashMap<i64, Box<dyn Object>>,
    light_lookup: HashMap<i64, Vector>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            object_lookup: HashMap::new(),
            light_lookup: HashMap::new(),
        }
    }

    fn get_object(&mut self, id: i64) -> Entry<i64, Box<dyn Object>> {
        self.object_lookup.entry(id)
    }

    fn add_object(&mut self, id: i64, obj: Box<dyn Object>) {
        self.object_lookup.insert(id, obj);
    }

    fn get_light_source(&mut self, id: i64) -> Entry<i64, Vector> {
        self.light_lookup.entry(id)
    }

    fn add_light_source(&mut self, id: i64, light: Vector) {
        self.light_lookup.insert(id, light);
    }

    fn get_reflected_light(&self, ray: &Ray) -> f64 {
        self.get_closest_object(ray, -1)
            .and_then(|obj| self.get_reflection(&ray, &obj))
            .unwrap_or(0.)
    }

    fn get_closest_object(&self, ray: &Ray, omit: i64) -> Option<&Box<dyn Object>> {
        self.object_lookup
            .iter()
            .filter_map(|(k, v)| if *k == omit { None } else { Some(v) })
            .map(|obj| {
                let obj_to_ray = obj.get_position().subtract(&ray.origin);
                if obj_to_ray.cos_between(&ray.dir()) < 0. {
                    return (obj, f64::INFINITY);
                }

                let Some(pt_int) = obj.get_point_of_intersection(&ray) else {
                    return (obj, f64::INFINITY);
                };

                return (obj, pt_int.subtract(&ray.origin).magnitude());
            })
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
            .and_then(|(obj, dist)| {
                if dist == f64::INFINITY {
                    None
                } else {
                    Some(obj)
                }
            })
    }

    fn get_reflection(&self, ray: &Ray, obj: &Box<dyn Object>) -> Option<f64> {
        // 1. Panic if the ray does not hit the sphere; we should have a guaranteed hit here
        let pt_int = obj.get_point_of_intersection(&ray).unwrap();

        // 2. Get the incident vector of the ray. Note that this is not normalized
        let ray_dir = ray.dir();

        // 3. Get the direction vector from the center of the sphere to the point of intersection
        let obj_norm = obj.get_normal_at_point(&pt_int);

        // 4. Compute the ray of reflection via formula 𝑟 = 𝑑 − 2(𝑑⋅𝑛)𝑛 where
        // 𝑑 = is the incident ray, 𝑛 is the normal vector of the surface
        let ray_of_reflection =
            ray_dir.subtract(&obj_norm.scalar_mult(ray_dir.dot_product(&obj_norm) * 2.));

        // 5. Accumulate the total light from multiples sources
        let mut total_light = 0.;
        for (_k, source) in self.light_lookup.iter() {
            // The direction of the light source relative to the point of intersection
            let light_dir = pt_int.subtract(source);

            // An approximation of how close in direction the reflected ray and the light source are from
            // the point of intersection
            let cos_ang = light_dir.cos_between(&ray_of_reflection);

            // Make sure the light source is "visible" from the point of intersection
            let Some(light_pt_on_sphere) =
                obj.get_point_of_intersection(&Ray::new(source.clone(), pt_int.clone()))
            else {
                continue;
            };

            // Due to numerical instability.
            if !light_pt_on_sphere.approx(&pt_int) {
                continue;
            }

            // If cos_ang is negative, the reflected ray is in the opposite direction of the light source
            // direction vector
            if cos_ang >= 0. {
                continue;
            }

            let reflected_ray = Ray::new(
                pt_int.add(&light_dir.normalize().scalar_mult(-1e-5)),
                source.clone(),
            );

            let is_blocked = self
                .get_closest_object(&reflected_ray, obj.id())
                .map(|closest_obj| {
                    let mag = closest_obj
                        .get_point_of_intersection(&reflected_ray)
                        .unwrap()
                        .subtract(&pt_int)
                        .magnitude();
                    mag < light_dir.magnitude()
                })
                .unwrap_or(false);

            if is_blocked {
                continue;
            }

            total_light += -cos_ang * 200. + 55.;
        }

        Some(total_light)
    }
}
