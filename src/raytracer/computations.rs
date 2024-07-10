use crate::tuple::Tuple;

/// Represents the various computations needed for shading an intersection point.
///
/// This struct encapsulates all the necessary geometric and material properties
/// at the point of intersection between a ray and an object in the scene. It includes
/// the distance to the intersection point, the object's ID, the point of intersection,
/// vectors for the eye direction, normal at the point, reflection vector, and whether
/// the intersection occurs inside the object. It also includes the over point for
/// shadow calculations, the under point for refraction calculations, and the refractive
/// indices before and after the intersection.
#[allow(dead_code)]
pub struct Computations {
    pub t: f64,             // The distance from the ray origin to the intersection point.
    pub object: usize,      // The ID of the object intersected by the ray.
    pub point: Tuple,       // The point of intersection.
    pub eyev: Tuple,        // The vector from the point of intersection towards the eye or camera.
    pub normalv: Tuple,     // The normal vector at the point of intersection.
    pub inside: bool,       // A boolean indicating if the intersection occurs inside the object.
    pub over_point: Tuple, // A point slightly above the surface at the point of intersection to avoid shadow acne.
    pub under_point: Tuple, // A point slightly below the surface at the point of intersection for refraction calculations.
    pub reflectv: Tuple,    // The reflection vector at the point of intersection.
    pub n1: f64,            // The refractive index of the medium the ray is coming from.
    pub n2: f64,            // The refractive index of the medium the ray is entering.
}

impl Computations {
    /// Calculates the Schlick approximation for the reflectance.
    ///
    /// This function computes the Schlick approximation, which is an estimation
    /// of the Fresnel effect to determine the reflectance based on the angle of
    /// incidence. It is used to mix the reflective and refractive components
    /// based on the viewing angle, enhancing realism in transparent materials.
    ///
    /// # Returns
    ///
    /// The reflectance as a `f64`, which is a value between 0 and 1 indicating
    /// the proportion of light reflected.
    pub fn schlick(&self) -> f64 {
        let mut cos = self.eyev.dot(&self.normalv);
        if self.n1 > self.n2 {
            let n = self.n1 / self.n2;
            let sin2_t = n * n * (1.0 - cos * cos);
            if sin2_t > 1.0 {
                return 1.0; // Total internal reflection.
            }

            let cos_t = (1.0 - sin2_t).sqrt();
            cos = cos_t;
        }

        let r0 = ((self.n1 - self.n2) / (self.n1 + self.n2)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cos).powi(5) // Reflectance with angle of incidence consideration.
    }
}
