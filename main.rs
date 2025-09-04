use std::fs::OpenOptions;
use std::io::prelude::*;

#[derive(Debug, Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Point {
    fn new(x: f64, y: f64, z: f64) -> Point {
        Point { x, y, z }
    }
}

struct Sphere {
    point: Point,
    r: f64,
}

struct Ray {
    origin: Point,
    dir_pt: Point,
}

impl Ray {
    fn new(origin: Point, dir_pt: Point) -> Ray {
        Ray { origin, dir_pt }
    }

    fn dir(&self) -> Point {
        self.dir_pt.subtract(&self.origin)
    }
}

impl Sphere {
    fn get_point_of_intersection(&self, ray: &Ray) -> Option<Point> {
        // The ray from the camera to the center of the self
        let dir = ray.dir();
        let hypDir = self.point.subtract(&ray.origin);
        let hypSq = hypDir.dot_product(&hypDir);

        let top = dir.dot_product(&hypDir);
        let cosSq = (top * top) / (dir.dot_product(&dir) * hypSq);

        let adjSq = cosSq * hypSq;
        let oppSq = hypSq - adjSq;

        let radSq = self.r * self.r;

        // If the ray is not within the bounds of the self, there is no collision
        if oppSq > radSq {
            return None;
        }

        let normRayDir = dir.normalize();
        let adj = f64::sqrt(adjSq);

        let distInt = adj - f64::sqrt(radSq - oppSq);
        // The point of intersection on the self
        Some(ray.origin.add(&normRayDir.scalar_mult(distInt)))
    }
}

fn main() {
    let num_frames: f64 = 50.;
    for i in 0..(num_frames as isize) {
        let x = f64::sin((i as f64 / num_frames) * 2. * std::f64::consts::PI);
        let y = f64::cos((i as f64 / num_frames) * 2. * std::f64::consts::PI);

        let pt = Point {
            x: 25000. * x,
            y: 25000.,
            z: 25000. * y,
        };

        capture(&pt, &format!("z_orbit/{}.ppm", i));
    }
}

fn capture(light: &Point, file_name: &str) {
    let mut screen = [[[255u8; 3]; 640]; 480];

    let sph = Sphere {
        point: Point::new(0., 0., 10000.),
        r: 125.,
    };

    for y in 0..screen.len() {
        for x in 0..screen[y].len() {
            let normX = x as f64 - (screen[0].len() as f64 / 2.);
            let normY = (screen.len() as f64 / 2.) - y as f64;

            let point = Point::new(normX, normY, 0.);
            let endPoint = Point::new(normX, normY, 1.0);
            let lightRay = Ray::new(point.clone(), endPoint.clone());

            let light = get_reflection(&lightRay, &[light], &sph);
            let cappedLight = light as u8;

            screen[y][x] = [cappedLight, cappedLight, cappedLight];
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

fn get_reflection(ray: &Ray, lightSources: &[&Point], sphere: &Sphere) -> f64 {
    let Some(ptInt) = sphere.get_point_of_intersection(&ray) else {
        return 0.;
    };

    let rayDir = ray.dir();

    // The direction from the center of the sphere to the point of intersection
    let sphNorm = ptInt.subtract(&sphere.point).normalize();

    let rayOfReflection = rayDir.subtract(&sphNorm.scalar_mult(rayDir.dot_product(&sphNorm) * 2.));

    let mut totalLight = 0.;
    for source in lightSources {
        let lightDir = ptInt.subtract(source);

        let cosAng = lightDir.dot_product(&rayOfReflection)
            / (rayOfReflection.magnitude() * lightDir.magnitude());

        if cosAng >= 0. {
            return 25.;
        }

        let Some(lightPtOnSphere) =
            sphere.get_point_of_intersection(&Ray::new((*source).clone(), ptInt.clone()))
        else {
            // Due to numerical instability.
            continue;
        };

        if !lightPtOnSphere.approx(&ptInt) {
            return 25.;
        }

        // We might be getting light at the opposite end of the sphere... we need
        // to check the direction of the light ray with respect to the reflection ray
        // and determine they're pointing in the right direction
        totalLight += -cosAng * 200. + 55.;
    }

    totalLight
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
        let sqSum = self.dot_product(self);

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

    fn approx(&self, other: &Point) -> bool {
        let eps: f64 = 10e-6;

        let is_approx = |val0, val1| f64::abs(val1 - val0) < eps;

        is_approx(self.x, other.x) && is_approx(self.y, other.y) && is_approx(self.z, other.z)
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
