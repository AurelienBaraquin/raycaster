use super::transform::CameraSpace;

/// Distance minimale devant la camera pour qu'un point soit considere visible.
/// Empeche une division par une profondeur nulle/negative a la projection.
const NEAR: f32 = 0.01;

/// Decoupe un segment (mur, en espace camera) contre le plan proche.
/// Retourne `None` si le segment est entierement derriere la camera,
/// sinon le segment (eventuellement raccourci) entierement visible.
pub fn clip_near(a: CameraSpace, b: CameraSpace) -> Option<(CameraSpace, CameraSpace)> {
    match (a.y > NEAR, b.y > NEAR) {
        (true, true) => Some((a, b)),
        (false, false) => None,
        (true, false) => Some((a, interpolate_at_near(a, b))),
        (false, true) => Some((interpolate_at_near(b, a), b)),
    }
}

/// Point sur le segment `from -> to` a l'endroit ou `y == NEAR`.
fn interpolate_at_near(from: CameraSpace, to: CameraSpace) -> CameraSpace {
    let t = (NEAR - from.y) / (to.y - from.y);
    CameraSpace {
        x: from.x + t * (to.x - from.x),
        y: NEAR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_fully_in_front_is_unchanged() {
        let a = CameraSpace { x: -1.0, y: 2.0 };
        let b = CameraSpace { x: 1.0, y: 3.0 };

        let result = clip_near(a, b);

        assert_eq!(result, Some((a, b)));
    }

    #[test]
    fn segment_fully_behind_is_discarded() {
        let a = CameraSpace { x: -1.0, y: -2.0 };
        let b = CameraSpace { x: 1.0, y: -3.0 };

        assert_eq!(clip_near(a, b), None);
    }

    #[test]
    fn segment_straddling_near_plane_is_shortened() {
        // a devant (y=2), b derriere (y=-2) -> le milieu du segment (en x) est
        // atteint a y=0, l'intersection avec NEAR (0.01) doit etre tres proche.
        let a = CameraSpace { x: -2.0, y: 2.0 };
        let b = CameraSpace { x: 2.0, y: -2.0 };

        let (clipped_a, clipped_b) = clip_near(a, b).expect("segment partiellement visible");

        assert_eq!(clipped_a, a);
        assert_eq!(clipped_b.y, NEAR);
        // a mi-chemin en y (2 -> -2 passe par NEAR proche de t=0.5) -> x proche de 0
        assert!((clipped_b.x - 0.0).abs() < 0.02);
    }
}
