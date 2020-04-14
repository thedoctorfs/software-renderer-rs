use nalgebra_glm::*;
/*void Camera::Move(float forward, float left) {
glm::vec3 direction = CalculateDirectionToMoveIn(around, updown);
glm::vec3 leftdirection(-direction.z, direction.y, direction.x);
position += direction * -forward;
position += leftdirection * left;
}
void Camera::MouseMove(float around_diff, float updown_diff) {
updown += updown_diff;
around += around_diff;
updown = std::min(pi_2 - 0.1f, std::max(-pi_2 + 0.1f, updown));
}
const glm::mat4 Camera::GetView() {
glm::mat4 arcBallRotation = ArcBallRotation(around, updown);
glm::vec3 directiontopos(glm::vec4(0.0f, 0.0f, -1.0f, 1.0f) * arcBallRotation);
glm::vec3 upvector(glm::vec4(0.0f, 1.0f, 0.0f, 1.0f) * arcBallRotation);
return glm::lookAt((glm::normalize(directiontopos) * glm::vec3(zoom, zoom, zoom)) + position, position, glm::normalize(upvector));
}
glm::vec3 Camera::CalculateDirectionToMoveIn(float around, float updown) {
glm::vec4 direction = glm::vec4(0.0f, 0.0f, -1.0f, 1.0f) * ArcBallRotation(around, updown);
direction.y = 0.0f; // remove y component, we are only interested in plane movement
return glm::normalize(glm::vec3(direction));
}
glm::mat4 Camera::ArcBallRotation(float around, float updown) {
return glm::rotate(glm::mat4(1.0f), updown, glm::vec3(1.0f, 0.0f, 0.0f)) *
glm::rotate(glm::mat4(1.0f), around, glm::vec3(0.0f, 1.0f, 0.0f));
}

glm::vec3 position = glm::vec3(0.0f, 5.0f, 0.0f);
float updown = 0.0f;
float around = 0.0f;
float zoom = 20.0f;
*/

pub struct Camera {
    initial_up: Vec3,
    initial_direction: Vec3,
    position: Vec3,
    orientation: Quat,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            initial_up: vec3(0.0, 1.0, 0.0),
            initial_direction: vec3(0.0, 0.0, -1.0),
            position: vec3(0.0, 0.0, 2.0),
            orientation: quat_identity(),
        }
    }
    fn up(&self) -> Vec3 {
        quat_rotate_vec3(&self.orientation, &self.initial_up)
    }

    fn direction(&self) -> Vec3 {
        quat_rotate_vec3(&self.orientation, &self.initial_direction)
    }

    fn right(&self) -> Vec3 {
        cross(&self.up(), &self.direction())
    }

    // rotate around right vector (cross product between up and direction
    pub fn pitch(&mut self, val: f32) {
        let pitch_q = quat_angle_axis(val, &self.right());
        self.orientation = &self.orientation * pitch_q;
    }

    // rotate around up vector
    pub fn yaw(&mut self, val: f32) {
        let yaw_q = quat_angle_axis(val, &self.up());
        self.orientation = &self.orientation * yaw_q;
    }

    // rotate around direction vector
    pub fn roll(&mut self, val: f32) {
        let roll_q = quat_angle_axis(val, &self.direction());
        self.orientation = &self.orientation * roll_q;
    }

    pub fn get_view(&self) -> Mat4 {
        //println!("{} {}", &self.up(), &self.direction());
        look_at(&self.position, &(&self.position + self.direction()), &self.up())
    }
}