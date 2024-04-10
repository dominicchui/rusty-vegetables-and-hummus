use nalgebra::{Matrix4, Rotation3, Vector3};

pub(crate) struct Camera {
    pub(crate) m_position: Vector3<f32>,
    m_pitch: f32,
    m_yaw: f32,
    pub(crate) m_look: Vector3<f32>,
    m_orbit_point: Vector3<f32>,
    _m_is_orbiting: bool,
    m_view: Matrix4<f32>,
    m_proj: Matrix4<f32>,
    m_view_dirty: bool,
    m_proj_dirty: bool,
    m_fov_y: f32,
    m_aspect: f32,
    m_near: f32,
    m_far: f32,
    _m_zoom: f32,
}

impl Camera {
    pub fn init() -> Self {
        Camera {
            m_position: Vector3::zeros(),
            m_pitch: 0.0,
            m_yaw: 0.0,
            m_look: Vector3::zeros(),
            m_orbit_point: Vector3::zeros(),
            _m_is_orbiting: false,
            m_view: Matrix4::identity(),
            m_proj: Matrix4::identity(),
            m_view_dirty: false,
            m_proj_dirty: false,
            m_fov_y: 0.0,
            m_aspect: 0.0,
            m_near: 0.0,
            m_far: 0.0,
            _m_zoom: 0.0,
        }
    }

    pub(crate) fn look_at(&mut self, eye: Vector3<f32>, target: Vector3<f32>) {
        self.m_position = eye;
        self.m_look = (target - eye).normalize();
        self.m_view_dirty = true;
        self.update_pitch_and_yaw();
    }

    pub(crate) fn set_orbit_point(&mut self, orbit_point: Vector3<f32>) {
        self.m_orbit_point = orbit_point;
        self.m_view_dirty = true;
    }

    pub(crate) fn set_perspective(&mut self, fov_y: f32, aspect: f32, near: f32, far: f32) {
        self.m_fov_y = fov_y;
        self.m_aspect = aspect;
        self.m_near = near;
        self.m_far = far;
        self.m_proj_dirty = true;
    }

    fn update_pitch_and_yaw(&mut self) {
        self.m_pitch = f32::asin(-self.m_look.y);
        self.m_yaw = f32::atan2(self.m_look.x, self.m_look.z);
    }

    pub(crate) fn get_view(&mut self) -> Matrix4<f32> {
        if self.m_view_dirty {
            let pos: Vector3<f32> = self.m_position;
            let look: Vector3<f32> = self.m_look;
            let up: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

            let mtrans = Matrix4::new(
                1.0, 0.0, 0.0, -pos.x, 0.0, 1.0, 0.0, -pos.y, 0.0, 0.0, 1.0, -pos.z, 0.0, 0.0, 0.0,
                1.0,
            );
            let w = -look.clone().normalize();
            let v = (up - up.dot(&w) * w).normalize();
            let u = v.cross(&w);

            let mrot = Matrix4::new(
                u.x, u.y, u.z, 0.0, v.x, v.y, v.z, 0.0, w.x, w.y, w.z, 0.0, 0.0, 0.0, 0.0, 1.0,
            );
            self.m_view = mrot * mtrans;
        }
        self.m_view
    }

    pub(crate) fn get_projection(&mut self) -> Matrix4<f32> {
        if self.m_proj_dirty {
            let theta = self.m_fov_y * 0.5;
            let inv_range = 1.0 / (self.m_far - self.m_near);
            let inv_tan = 1.0 / f32::tan(theta);
            self.m_proj[0] = inv_tan / self.m_aspect;
            self.m_proj[5] = inv_tan;
            self.m_proj[10] = -(self.m_near + self.m_far) * inv_range;
            self.m_proj[11] = -1.0;
            self.m_proj[14] = -2.0 * self.m_near * self.m_far * inv_range;
            self.m_proj[15] = 0.0;
            self.m_proj_dirty = false;
        }
        self.m_proj
    }

    pub(crate) fn move_camera(&mut self, delta_pos: Vector3<f32>) {
        if delta_pos.norm_squared() == 0.0 {
            return;
        }

        self.m_position += delta_pos;

        // if (m_isOrbiting) {
        //     m_orbitPoint += deltaPosition;
        // }

        self.m_view_dirty = true;
    }

    pub(crate) fn rotate_camera(&mut self, angle: f32) {
        // println!("rotate by {}", angle);
        // rotate around z-axis (z-up)
        // let axis = Vector3::z_axis();
        // let rot = Rotation3::from_axis_angle(&axis, angle);
        // self.m_look = rot * self.m_look;
        // self.update_pitch_and_yaw();
        // self.m_view_dirty = true;
    }
}
