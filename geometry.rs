#[derive(Debug, Copy, Clone)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    pub fn zero() -> Vector {
        Vector {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn subtract(&self, other: &Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn dot_product(&self, other: &Vector) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    pub fn cross_product(&self, other: &Vector) -> Vector {
        Vector {
            x: self.y * other.z - self.z * other.y,
            y: -(self.x * other.z - self.z * other.x),
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn magnitude(&self) -> f64 {
        let sq_sum = self.dot_product(self);

        f64::sqrt(sq_sum)
    }

    pub fn normalize(&self) -> Vector {
        let mag = self.magnitude();

        Vector {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    pub fn approx(&self, other: &Vector) -> bool {
        let eps: f64 = 10e-6;

        let is_approx = |val0, val1| f64::abs(val1 - val0) < eps;

        is_approx(self.x, other.x) && is_approx(self.y, other.y) && is_approx(self.z, other.z)
    }

    pub fn add(&self, other: &Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn scalar_mult(&self, scalar: f64) -> Vector {
        Vector {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }

    pub fn cos_between(&self, other: &Vector) -> f64 {
        self.dot_product(&other) / (self.magnitude() * other.magnitude())
    }

    pub fn swizzle(&self) -> Vector {
        Vector {
            x: self.y,
            y: self.z,
            z: self.x,
        }
    }

    pub fn unswizzle(&self) -> Vector {
        Vector {
            x: self.z,
            y: self.x,
            z: self.y,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vector,
    pub dir_pt: Vector,
}

impl Ray {
    pub fn new(origin: Vector, dir_pt: Vector) -> Ray {
        Ray { origin, dir_pt }
    }

    pub fn dir(&self) -> Vector {
        self.dir_pt.subtract(&self.origin)
    }

    pub fn normalize(&self) -> Ray {
        Ray {
            origin: self.origin,
            dir_pt: self.origin.add(&self.dir().normalize()),
        }
    }
}

pub trait Object {
    fn get_point_of_intersection(&self, ray: &Ray) -> Option<Vector>;
    fn get_normal_at_point(&self, v: &Vector) -> Vector;

    fn get_position(&self) -> Vector;
    fn set_position(&mut self, new: Vector);

    fn id(&self) -> i64;
}

pub struct Sphere {
    pub point: Vector,
    pub r: f64,

    pub id: i64,
}

impl Sphere {
    pub fn new(point: Vector, r: f64, id: i64) -> Sphere {
        Sphere { point, r, id }
    }
}

impl Object for Sphere {
    fn get_point_of_intersection(&self, ray: &Ray) -> Option<Vector> {
        // The ray from the camera to the center of the self
        let dir = ray.dir();
        let hyp_dir = self.point.subtract(&ray.origin);
        let hyp_sq = hyp_dir.dot_product(&hyp_dir);

        let top = dir.dot_product(&hyp_dir);
        let cos_sq = (top * top) / (dir.dot_product(&dir) * hyp_sq);

        let adj_sq = cos_sq * hyp_sq;
        let opp_sq = hyp_sq - adj_sq;

        let rad_sq = self.r * self.r;

        // If the ray is not within the bounds of the self, there is no collision
        if opp_sq > rad_sq {
            return None;
        }

        let norm_ray_dir = dir.normalize();
        let adj = f64::sqrt(adj_sq);

        let dist_int = adj - f64::sqrt(rad_sq - opp_sq);

        // The point of intersection on the self
        Some(ray.origin.add(&norm_ray_dir.scalar_mult(dist_int)))
    }

    fn get_normal_at_point(&self, v: &Vector) -> Vector {
        v.subtract(&self.point).normalize()
    }

    fn get_position(&self) -> Vector {
        self.point
    }

    fn set_position(&mut self, new: Vector) {
        self.point = new;
    }

    fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub p0: Vector,
    pub p1: Vector,
    pub p2: Vector,
    pub m: f64,
    pub n: f64,
    pub k: f64,
    pub l: f64,
    pub id: i64,
}

impl Plane {
    pub fn new(p0: Vector, p1: Vector, p2: Vector, id: i64) -> Plane {
        let [mut p0n, mut p1n, mut p2n] = [p0.normalize(), p1.normalize(), p2.normalize()];
        if p0n.x == p1n.x {
            assert_ne!(p1n.x, p2n.x);
            [p0n, p2n] = [p2n, p0n]
        } else if p0n.x == p2n.x {
            assert_ne!(p1n.x, p2n.x);
            [p0n, p1n] = [p1n, p0n]
        }
        // TODO: check that the points are not in a line
        Plane {
            p0: p0,
            p1: p1,
            p2: p2,
            m: (p1n.y - p0n.y) / (p1n.x - p0n.x),
            n: (p1n.z - p0n.z) / (p1n.x - p0n.x),
            k: (p2n.y - p0n.y) / (p2n.x - p0n.x),
            l: (p2n.z - p0n.z) / (p2n.x - p0n.x),
            id,
        }
    }
}

const COMPARISON_EPSILON: f64 = 1e-8;

impl Object for Plane {
    fn get_point_of_intersection(&self, ray: &Ray) -> Option<Vector> {
        // First check if the line is parallel.
        let raynorm = ray.normalize();
        let arbitrary_plane_line = self.p0.subtract(&self.p1).normalize();
        if arbitrary_plane_line
            .cross_product(&raynorm.dir())
            .magnitude()
            < COMPARISON_EPSILON
        {
            return None;
        }

        // We chose x as our division axis, we swap axes if our ray is x axis aligned.
        let raydir = raynorm.dir();
        if raydir.x == 0. {
            let swizzled_ray = Ray {
                origin: ray.origin.swizzle(),
                dir_pt: ray.dir_pt.swizzle(),
            };
            let swizzled_plane =
                Plane::new(self.p0.swizzle(), self.p1.swizzle(), self.p2.swizzle(), -1);
            if let Some(swizzled_intersection_pt) =
                swizzled_plane.get_point_of_intersection(&swizzled_ray)
            {
                return Some(swizzled_intersection_pt.unswizzle());
            } else {
                return None;
            }
        }

        let h = raydir.y / raydir.x;
        let i = raydir.z / raydir.x;
        let [a, b, c] = [ray.origin.x, ray.origin.y, ray.origin.z];

        // Now check for degenerate divide by zero cases.

        let denominator = self.k - h - (self.l - i) * (self.k + self.m) / (self.l + self.n);
        let numerator1part = self.p0.y + b - self.p0.x * self.m - a * h;
        let numerator2part =
            (self.p0.z + c - self.p0.x * self.n - a * i) * (self.k + self.m) / (self.l + self.n);
        let xval = (numerator1part - numerator2part) / denominator;
        return Some(Vector {
            x: xval,
            y: (xval - a) * h + b,
            z: (xval - a) * i + c,
        });
    }

    fn get_normal_at_point(&self, v: &Vector) -> Vector {
        // This doesn't necessarily point in the correct direction
        self.p1
            .subtract(&self.p0)
            .cross_product(&self.p2.subtract(&self.p0))
            .normalize()
    }

    fn get_position(&self) -> Vector {
        self.p0
    }

    fn set_position(&mut self, new: Vector) {
        // This API doesn't handle rotations!
        let delta = new.subtract(&self.p0);
        self.p0 = new;
        self.p1 = self.p1.add(&delta);
        self.p2 = self.p2.add(&delta);
    }

    fn id(&self) -> i64 {
        self.id
    }
}
