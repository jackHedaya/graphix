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

            let light = get_reflection(
                (&point, &endPoint),
                &[Point {
                    x: 50.,
                    y: 50.,
                    z: 50.,
                }],
                &sph,
            );
            let cappedLight = light as u8;

            screen[y][x] = [cappedLight, cappedLight, cappedLight];
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

fn get_reflection(ray: (&Point, &Point), lightSources: &[Point], sphere: &Sphere) -> f64 {
    // The directional ray sent from the camera
    let rayDir = ray.1.subtract(&ray.0);
    // The ray from the camera to the center of the sphere
    let hypDir = sphere.point.subtract(ray.0);
    let hypSq = hypDir.dot_product(&hypDir);

    let top = rayDir.dot_product(&hypDir);
    let cosSq = (top * top) / (rayDir.dot_product(&rayDir) * hypSq);

    let adjSq = cosSq * hypSq;
    let oppSq = hypSq - adjSq;

    let radSq = sphere.r * sphere.r;

    // If the ray is not within the bounds of the sphere, there is no collision and the object
    // returns no light
    if oppSq > radSq {
        return 0.;
    }

    let normRayDir = rayDir.normalize();
    let adj = f64::sqrt(adjSq);

    let distInt = adj - f64::sqrt(radSq - oppSq);
    // The point of intersection on the sphere
    let ptInt = ray.0.add(&normRayDir.scalar_mult(distInt));

    // The direction from the center of the sphere to the point of intersection
    let sphNorm = ptInt.subtract(&sphere.point).normalize();

    let rayOfReflection = rayDir.subtract(&sphNorm.scalar_mult(rayDir.dot_product(&sphNorm) * 2.));

    let mut totalLight = 0.;
    for source in lightSources {
        let lightDir = ptInt.subtract(source);

        let cosAng = lightDir.dot_product(&rayOfReflection)
            / (rayOfReflection.magnitude() * lightDir.magnitude());

        // We might be getting light at the opposite end of the sphere... we need
        // to check the direction of the light ray with respect to the reflection ray
        // and determine they're pointing in the right direction
        totalLight += cosAng * 255.;
    }

    totalLight
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

    fn magnitude(&self) -> f64 {
        let sqSum = self.dot_product(&self);

        f64::sqrt(sqSum)
    }

    fn normalize(&self) -> Point {
        let mag = self.magnitude();

        Point {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    fn add(&self, other: &Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn scalar_mult(&self, scalar: f64) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}
