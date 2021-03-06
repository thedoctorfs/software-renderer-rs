use bevy::prelude::*;

pub fn calculate_low_high(point0: Vec2, point1: Vec2) -> (Vec2, Vec2) {
    if point0.x <= point1.x && point0.y <= point1.y {
        (Vec2::new(point0.x, point0.y), Vec2::new(point1.x, point1.y))
    } else if point0.x <= point1.x && point0.y > point1.y {
        (Vec2::new(point0.x, point1.y), Vec2::new(point1.x, point0.y))
    } else if point0.x > point1.x && point0.y <= point1.y {
        (Vec2::new(point1.x, point0.y), Vec2::new(point0.x, point1.y))
    } else {
        (Vec2::new(point1.x, point1.y), Vec2::new(point0.x, point0.y))
    }
}

pub fn calculate_midpoint_scale(point0: Vec2, point1: Vec2) -> (Vec2, Vec2) {
    let (low, high) = calculate_low_high(point0, point1);

    let midpoint = (low + high) / 2.0;
    let scale = high - low;
    (midpoint, scale)
}

pub fn is_selected(low: Vec2, high: Vec2, point: Vec2) -> bool {
    let selection_margin = 0.5;
    assert!(low.x <= high.x);
    assert!(low.y <= high.y);
    point.x > low.x - selection_margin
        && point.x < high.x + selection_margin
        && point.y > low.y - selection_margin
        && point.y < high.y + selection_margin
}
