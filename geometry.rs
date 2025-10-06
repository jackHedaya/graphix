#[derive(Debug, Copy, Clone)]
pub struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
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
}

pub struct Ray {
    origin: Vector,
    dir_pt: Vector,
}

impl Ray {
    pub fn new(origin: Vector, dir_pt: Vector) -> Ray {
        Ray { origin, dir_pt }
    }

    pub fn dir(&self) -> Vector {
        self.dir_pt.subtract(&self.origin)
    }
}

pub trait Object {
    fn get_point_of_intersection(&self, ray: &Ray) -> Option<Vector>;
}

pub struct Sphere {
  pub point: Vector,
  pub r: f64,
}

impl Sphere {
    pub fn new(point: Vector, r: f64) -> Sphere {
        Sphere { point, r }
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
}
