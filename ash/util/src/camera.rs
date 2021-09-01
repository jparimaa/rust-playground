pub const FORWARD: cgmath::Vector4<f32> = cgmath::Vector4 {
    x: 0.0,
    y: 0.0,
    z: -1.0,
    w: 0.0,
};

pub const LEFT: cgmath::Vector4<f32> = cgmath::Vector4 {
    x: -1.0,
    y: 0.0,
    z: 0.0,
    w: 0.0,
};

pub struct Camera {
    mouse_enabled: bool,
    sensitivity: f32,
    last_mouse_x: i32,
    last_mouse_y: i32,
    rot_x: f32,
    rot_y: f32,
    //
    movement_speed: f32,
    forward_movement: f32,
    strafe_movement: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    pos: cgmath::Vector3<f32>,
    //
    fovy_deg: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(sensitivity: f32, movement_speed: f32, aspect: f32) -> Camera {
        Camera {
            mouse_enabled: false,
            sensitivity,
            last_mouse_x: 0,
            last_mouse_y: 0,
            rot_x: 0.0,
            rot_y: 0.0,
            movement_speed,
            forward_movement: 0.0,
            strafe_movement: 0.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            pos: cgmath::Vector3 { x: 0.0, y: 0.0, z: 2.0 },
            fovy_deg: 45.0,
            aspect,
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn update(&mut self, time_delta: f32) {
        self.pos += time_delta * self.movement_speed * self.forward_movement * self.get_forward_vector();
        self.pos += time_delta * self.movement_speed * self.strafe_movement * self.get_left_vector();
    }

    pub fn handle_key_input(&mut self, key: winit::event::VirtualKeyCode, state: winit::event::ElementState) {
        use winit::event::ElementState;
        use winit::event::VirtualKeyCode;

        match key {
            VirtualKeyCode::W => self.move_forward = state == ElementState::Pressed,
            VirtualKeyCode::S => self.move_backward = state == ElementState::Pressed,
            VirtualKeyCode::A => self.move_left = state == ElementState::Pressed,
            VirtualKeyCode::D => self.move_right = state == ElementState::Pressed,
            VirtualKeyCode::E => {
                if state == ElementState::Released {
                    self.mouse_enabled = !self.mouse_enabled;
                }
            }
            _ => {}
        }
        if self.move_forward && !self.move_backward {
            self.forward_movement = 1.0;
        } else if self.move_backward && !self.move_forward {
            self.forward_movement = -1.0;
        } else {
            self.forward_movement = 0.0;
        }
        if self.move_left && !self.move_right {
            self.strafe_movement = 1.0;
        } else if self.move_right && !self.move_left {
            self.strafe_movement = -1.0;
        } else {
            self.strafe_movement = 0.0;
        }
    }

    pub fn handle_mouse_movement(&mut self, x: i32, y: i32) {
        if !self.mouse_enabled {
            self.last_mouse_x = x;
            self.last_mouse_y = y;
            return;
        }
        let delta_x = (x - self.last_mouse_x) as f32;
        let delta_y = (y - self.last_mouse_y) as f32;
        self.last_mouse_x = x;
        self.last_mouse_y = y;
        self.rot_x -= delta_x * self.sensitivity;
        self.rot_y += delta_y * self.sensitivity;
        self.rot_y = self.rot_y.min(85.0).max(-85.0);
    }

    pub fn get_view_matrix(&mut self) -> cgmath::Matrix4<f32> {
        use cgmath::{Matrix4, SquareMatrix};
        let translation = Matrix4::from_translation(self.pos);
        let v = translation * self.get_rotation_matrix();
        v.invert().unwrap()
    }

    pub fn get_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        use cgmath::{Deg, Rad};
        cgmath::perspective(Rad::from(Deg(self.fovy_deg)), self.aspect, self.near, self.far)
    }

    pub fn get_forward_vector(&self) -> cgmath::Vector3<f32> {
        let v4 = self.get_rotation_matrix() * FORWARD;
        cgmath::Vector3 { x: v4.x, y: v4.y, z: v4.z }
    }

    pub fn get_left_vector(&self) -> cgmath::Vector3<f32> {
        let v4 = self.get_rotation_matrix() * LEFT;
        cgmath::Vector3 { x: v4.x, y: v4.y, z: v4.z }
    }

    fn get_rotation_matrix(&self) -> cgmath::Matrix4<f32> {
        use cgmath::{Deg, Matrix4, Rad};
        let pitch = Matrix4::from_angle_x(Rad::from(Deg(self.rot_y)));
        let yaw = Matrix4::from_angle_y(Rad::from(Deg(self.rot_x)));
        yaw * pitch
    }
}
