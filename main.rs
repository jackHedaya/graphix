mod geometry;

use std::fs::{self, OpenOptions};
use std::io::prelude::*;

use crate::geometry::{Ray, Sphere, Vector};

const OUT_PATH: &str = "out";

fn main() {
    fs::create_dir_all(OUT_PATH).expect("failed to initialize output directory");

    let num_frames: f64 = 50.;
    for i in 0..(num_frames as isize) {
        let x = f64::sin((i as f64 / num_frames) * 2. * std::f64::consts::PI);
        let y = f64::cos((i as f64 / num_frames) * 2. * std::f64::consts::PI);

        let pts = [
            Vector::new(25000. * x, 25000., 25000. * y),
            Vector::new(25000. * -x, -25000., 25000. * y),
        ];

        capture(&pts, &format!("{}/{}.ppm", OUT_PATH, i));
    }
}

fn capture(light_sources: &[Vector], file_name: &str) {
    let mut screen = [[[255u8; 3]; 640]; 480];

    let sph = Sphere {
        point: Vector::new(0., 0., 10000.),
        r: 125.,
    };

    for y in 0..screen.len() {
        for x in 0..screen[y].len() {
            let norm_x = x as f64 - (screen[0].len() as f64 / 2.);
            let norm_y = (screen.len() as f64 / 2.) - y as f64;

            let point = Vector::new(norm_x, norm_y, 0.);
            let end_point = Vector::new(norm_x, norm_y, 1.0);
            let light_ray = Ray::new(point.clone(), end_point.clone());

            let light = get_reflection(&light_ray, light_sources, &sph);
            let capped_light = light as u8;

            screen[y][x] = [capped_light, capped_light, capped_light];
        }
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)
        .expect("");

    file.write_all(b"P6 640 480 255\n").expect("");

    for y in 0..screen.len() {
        for x in 0..screen[y].len() {
            file.write_all(&screen[y][x]).expect("");
        }
    }
}

fn get_reflection(ray: &Ray, light_sources: &[Vector], sphere: &Sphere) -> f64 {
    // 1. Early return if the ray doesn't hit the sphere at all
    let Some(pt_int) = sphere.get_point_of_intersection(&ray) else {
        return 0.;
    };

    // 2. Get the incident vector of the ray. Note that this is not normalized
    let ray_dir = ray.dir();

    // 3. Get the direction vector from the center of the sphere to the point of intersection
    let sph_norm = pt_int.subtract(&sphere.point).normalize();

    // 4. Compute the ray of reflection via formula 𝑟 = 𝑑 − 2(𝑑⋅𝑛)𝑛 where
    // 𝑑 = is the incident ray, 𝑛 is the normal vector of the surface
    let ray_of_reflection =
        ray_dir.subtract(&sph_norm.scalar_mult(ray_dir.dot_product(&sph_norm) * 2.));

    // 5. Accumulate the total light from multiples sources
    let mut total_light = 0.;
    for source in light_sources {
        // Make sure the light source is "visible" from the point of intersection
        let Some(light_pt_on_sphere) =
            sphere.get_point_of_intersection(&Ray::new((*source).clone(), pt_int.clone()))
        else {
            // Due to numerical instability.
            continue;
        };

        if !light_pt_on_sphere.approx(&pt_int) {
            continue;
        }

        // The direction of the light source relative to the point of intersection
        let light_dir = pt_int.subtract(source);

        // An approximation of how close in direction the reflected ray and the light source are from
        // the point of intersection
        let cos_ang = light_dir.dot_product(&ray_of_reflection)
            / (ray_of_reflection.magnitude() * light_dir.magnitude());

        // If cos_ang is negative, the reflected ray is in the opposite direction of the light source
        // direction vector
        if cos_ang >= 0. {
            continue;
        }

        total_light += -cos_ang * 200. + 55.;
    }

    total_light
}
