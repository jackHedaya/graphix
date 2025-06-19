use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
}

struct Sphere {
    point: Point,
    r: f64,
}

fn main() {
    let mut screen = [[[255u8; 3]; 640]; 480];

    let sph = Sphere {
        point: Point {
            x: 0.,
            y: 0.,
            z: 100.,
        },
        r: 125.,
    };

    for y in 0..screen.len() {
        for x in 0..screen[y].len() {
            let normX = x as f64 - (screen[0].len() as f64 / 2.);
            let normY = (screen.len() as f64 / 2.) - y as f64;

            let point = Point {
                x: normX,
                y: normY,
                z: 0.,
            };
            let endPoint = Point {
                x: normX,
                y: normY,
                z: 1.0,
            };

            if check_intersect((&point, &endPoint), &sph) {
                screen[y][x] = [200u8, 0u8, 0u8];
            }
        }
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("file.ppm")
        .expect("");

    file.write_all(b"P6 640 480 255\n").expect("");

    for y in 0..screen.len() {
        for x in 0..screen[y].len() {
            file.write_all(&screen[y][x]).expect("");
        }
    }
}

fn check_intersect(ray: (&Point, &Point), sphere: &Sphere) -> bool {
    let rayDir = ray.1.subtract(&ray.0);
    let hypDir = sphere.point.subtract(ray.0);
    let hypSq = hypDir.dot_product(&hypDir);
    let top = rayDir.dot_product(&hypDir);
    let cosSq = (top * top) / (rayDir.dot_product(&rayDir) * hypSq);

    let adjSq = cosSq * hypSq;
    let oppSq = hypSq - adjSq;

    debug(
        ray.0,
        format!("{rayDir:?} {hypDir:?} {hypSq} {cosSq} {adjSq} {oppSq}"),
    );

    oppSq < sphere.r * sphere.r
}

fn debug(point: &Point, msg: String) {
    if point.x == 0. && point.y == 0.0 && point.z == 0. {
        println!("{}", msg);
    }
}

impl Point {
    fn subtract(&self, other: &Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn dot_product(&self, other: &Point) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }
}
