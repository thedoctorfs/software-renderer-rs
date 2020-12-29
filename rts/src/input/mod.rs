use crate::client;
use bevy::{
    input::{mouse::MouseButtonInput, system::exit_on_esc_system, ElementState},
    prelude::*,
    render::camera::Camera,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(input_system.system())
            .add_system(exit_on_esc_system.system());
    }
}

#[derive(Default)]
struct State {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

fn input_system(
    mut state: Local<State>,
    mouse_button_events: Res<Events<MouseButtonInput>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    mut controllers: Query<(&mut client::CameraController, &mut client::PlayerController)>,
    camera: Query<(&GlobalTransform, &Camera)>,
) {
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());
    let (border_margin_width, border_margin_height) = (width / 10.0, height / 10.0);
    let mut current_position: Option<Vec2> = None;
    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        current_position = Some(event.position);
    }

    let mut place_object: Option<Vec3> = None;
    for (view, camera) in camera.iter() {
        if let Some(current_position) = current_position {
            let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);

            let cursor_pos_ndc: Vec3 =
                ((current_position / screen_size) * 2.0 - Vec2::from([1.0, 1.0])).extend(1.0);

            let camera_matrix = view.compute_matrix();

            let ndc_to_world = camera_matrix * camera.projection_matrix.inverse();
            let cursor_position = ndc_to_world.transform_point3(cursor_pos_ndc);
            place_object = Some(cursor_position);
        }
    }

    for (mut camera_controller, mut player_controller) in controllers.iter_mut() {
        if let Some(current_position) = current_position {
            let x = if current_position.x > width - border_margin_width {
                1.0
            } else if current_position.x < border_margin_width {
                -1.0
            } else {
                0.0
            };
            let y = if current_position.y > height - border_margin_height {
                1.0
            } else if current_position.y < border_margin_height {
                -1.0
            } else {
                0.0
            };
            camera_controller.move_position = Some(Vec2::new(x, y));
        }
        for event in state.mouse_button_event_reader.iter(&mouse_button_events) {
            match event {
                MouseButtonInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                } => player_controller.place_object = place_object,
                MouseButtonInput {
                    button: MouseButton::Left,
                    state: ElementState::Released,
                } => (),
                _ => (),
            }
        }
    }
}