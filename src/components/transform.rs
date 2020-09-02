use na::{Complex, Isometry2, Isometry3, Matrix4, UnitQuaternion, Vector3};

#[derive(Clone, Debug)]
pub struct Transform {
    pub isometry: Isometry3<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Transform::default()
    }
    pub fn with_translation<T: Into<Vector3<f32>>>(mut self, pos: T) -> Self {
        self.isometry.translation = pos.into().into();
        self
    }
    pub fn with_scale<T: Into<Vector3<f32>>>(mut self, scale: T) -> Self {
        self.scale = scale.into();
        self
    }
    pub fn with_rotation<T: Into<f32>>(mut self, rads: T) -> Self {
        self.isometry.rotation = UnitQuaternion::from_scaled_axis(Vector3::z() * rads.into());
        self
    }

    pub fn get_matrix(&self) -> Matrix4<f32> {
        self.isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    pub fn set_isometry(&mut self, val: Isometry3<f32>) {
        self.isometry = val;
    }

    pub fn set_isometry_2d(&mut self, val: Isometry2<f32>) {
        let translation = Vector3::new(val.translation.vector.x, val.translation.vector.y, 0.);
        let rotation = UnitQuaternion::from_scaled_axis(Vector3::z() * val.rotation.angle());
        self.isometry = Isometry3::from_parts(translation.into(), rotation);
    }
    pub fn as_2d(&self) -> Isometry2<f32> {
        Isometry2::new(
            self.isometry.translation.vector.xy(),
            self.isometry.rotation.angle(),
        )
    }

    pub fn move_left(&mut self, val: f32) {
        self.isometry.translation.x -= val;
    }

    pub fn move_right(&mut self, val: f32) {
        self.isometry.translation.x += val;
    }

    pub fn rotate_left(&mut self, val: f32) {
        self.isometry.rotation *= UnitQuaternion::from_scaled_axis(Vector3::z() * val);
    }

    pub fn rotate_right(&mut self, val: f32) {
        self.isometry.rotation *= UnitQuaternion::from_scaled_axis(Vector3::z() * -val);
    }
}

impl Default for Transform {
    fn default() -> Self {
        let translation = Vector3::new(0., 0., 0.);
        let rot = UnitQuaternion::from_scaled_axis(Vector3::zeros());
        Transform {
            isometry: Isometry3::from_parts(translation.into(), rot),
            scale: Vector3::new(1., 1., 1.),
        }
    }
}
