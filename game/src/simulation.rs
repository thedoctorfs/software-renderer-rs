use crate::client::command::FrameCommand;
use crate::graphics::clipmap;
use crate::input::player_input_state::{ForwardMovement, StrafeMovement};
use crate::{entity, transformation};
use nalgebra_glm::vec2;
use xp_physics::{collision_response_non_trianulated, Response, Sphere};

pub fn handle_frame(
    frame_commands: Vec<FrameCommand>,
    player: &mut entity::Entity,
    frame_time: f32,
    clipmap_renderer: &clipmap::Renderable,
) {
    for frame_command in frame_commands {
        if let Some(orientation_change) = frame_command.command.orientation_change {
            player.orientation = transformation::rotate_around_local_axis(
                &player.orientation,
                0.0,
                orientation_change.horizontal,
                0.0,
            )
        }
        let forward = match frame_command.command.forward {
            Some(ForwardMovement::Positive) => frame_time * player.velocity,
            Some(ForwardMovement::Negative) => frame_time * -player.velocity,
            None => 0.0,
        };
        let right = match frame_command.command.strafe {
            Some(StrafeMovement::Right) => frame_time * player.velocity,
            Some(StrafeMovement::Left) => frame_time * -player.velocity,
            None => 0.0,
        };
        let player_movement =
            transformation::move_along_local_axis(&player.orientation, forward, right, 0.0);
        let sphere_diameter = 2.0;
        let triangles = clipmap_renderer.create_triangle_mesh_around(
            &[
                vec2(player.position.x, player.position.z),
                vec2(
                    player.position.x + player_movement.x,
                    player.position.z + player_movement.z,
                ),
            ],
            sphere_diameter,
        );

        // detect collision player movement
        let response = collision_response_non_trianulated(
            Response {
                sphere: Sphere {
                    c: player.position,
                    r: 1.0,
                },
                movement: player_movement,
            },
            triangles.as_slice(),
        );
        /*
        let gravity_movement = vec3(0.0, -1.0, 0.0) * (3.0 * frame_time);

        // detect collision gravity (constant speed of 20 m/s TODO: fix this to 9.81 m/s2
        let response = collision_response_non_trianulated(
            Response {
                sphere: Sphere {
                    c: response.sphere.c + response.movement,
                    r: 1.0,
                },
                movement: gravity_movement,
            },
            triangles.as_slice(),
        );*/
        player.position = response.sphere.c + response.movement;
    }
}
