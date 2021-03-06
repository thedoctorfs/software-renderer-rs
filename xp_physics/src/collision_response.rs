use crate::response::Response;
use crate::triangle::plane_constant;
use crate::{Collision, Sphere};
use nalgebra_glm::dot;

pub fn sphere_triangle_calculate_response(response: &Response, collision: &Collision) -> Response {
    let sphere = &response.sphere;
    let movement = &response.movement;
    // The paper adjusts the sliding plane VERY_CLOSE_DISTANCE in front of the actual collision
    // but it does so moving in the direction of the sphere center, so the sliding plane can still be
    // very close (if the movement is almost parallel to the the plane of collision.
    // Also we cannot move in another direction then the center of the sphere.
    // Therefore we will not adjust, we try to fix this in the collision detection step, and make sure we do
    // adjust for floating point errors there. So no adjusting here.
    let slide_plane_origin = collision.intersection;
    let slide_plane_normal = nalgebra_glm::normalize(&(collision.position - slide_plane_origin));

    let original_destination = sphere.c + movement;

    let original_destination_to_plane_distance = dot(&original_destination, &slide_plane_normal)
        + plane_constant(&slide_plane_origin, &slide_plane_normal);
    assert!(original_destination_to_plane_distance >= 0.0);

    let new_destination =
        original_destination - original_destination_to_plane_distance * slide_plane_normal;
    let new_movement = new_destination - collision.position;

    Response {
        sphere: Sphere {
            c: new_destination,
            r: 1.0,
        },
        movement: new_movement,
    }
}
