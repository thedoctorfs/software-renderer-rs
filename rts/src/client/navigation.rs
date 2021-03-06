use bevy::prelude::Vec2;
use std::collections::VecDeque;

#[derive(PartialEq, Eq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct FlowField {
    values: Vec<u32>,
    flow: Vec<IVec2>,
    width: usize,
    height: usize,
}

impl Into<Cell> for (usize, usize) {
    fn into(self) -> Cell {
        Cell {
            x: self.0,
            y: self.1,
        }
    }
}

impl FlowField {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            values: vec![std::u32::MAX - 1; width * height],
            flow: vec![IVec2::zero(); width * height],
            width,
            height,
        }
    }

    pub fn reset(&mut self) {
        for v in &mut self.values {
            if *v != std::u32::MAX {
                *v = std::u32::MAX - 1;
            }
        }
    }

    pub fn set_blocked_cell(&mut self, cell: &Cell) {
        self.set(&cell, std::u32::MAX);
    }

    pub fn block_position_with_size(&mut self, pos: &IVec2, size: usize) {
        assert!(size % 2 == 0);
        let cell_position = self.position_to_cell(&pos);
        let half_size = size / 2;

        for y in (cell_position.y - half_size)..(cell_position.y + half_size) {
            for x in (cell_position.x - half_size)..(cell_position.x + half_size) {
                self.set_blocked_cell(&Cell::new(x, y));
            }
        }
    }

    pub fn get(&self, cell: &Cell) -> u32 {
        assert!(cell.x < self.width);
        assert!(cell.y < self.height);
        self.values[self.height * cell.y + cell.x]
    }

    pub fn set(&mut self, cell: &Cell, value: u32) {
        assert!(cell.x < self.width);
        assert!(cell.y < self.height);
        self.values[self.height * cell.y + cell.x] = value;
    }

    pub fn set_flow_cell(&mut self, x: usize, y: usize, direction: IVec2) {
        assert!(x < self.width);
        assert!(y < self.height);
        self.flow[self.height * y + x] = direction;
    }

    pub fn get_flow_cell(&self, x: usize, y: usize) -> IVec2 {
        assert!(x < self.width);
        assert!(y < self.height);
        self.flow[self.height * y + x]
    }

    pub fn get_flow_cell_f32(&self, x: usize, y: usize) -> Vec2 {
        let v = self.get_flow_cell(x, y);
        Vec2::new(v.x as f32, v.y as f32)
    }

    pub fn get_flow_bilininterpol(&self, position: &Vec2) -> Vec2 {
        let floor_pos = position.floor();
        let (x, y) = self.map(floor_pos.x as isize, floor_pos.y as isize);
        let f00 = self.get_flow_cell_f32(x, y);
        let f01 = self.get_flow_cell_f32(x, y + 1);
        let f10 = self.get_flow_cell_f32(x + 1, y);
        let f11 = self.get_flow_cell_f32(x + 1, y + 1);
        let mapped_position = self.mapped_position(position);
        let x_weight = mapped_position.x - x as f32;
        let top = f00 * (1.0 - x_weight) + f10 * x_weight;
        let bottom = f01 * (1.0 - x_weight) + f11 * x_weight;
        let y_weight = mapped_position.y - y as f32;
        let direction = (top * (1.0 - y_weight) + bottom * y_weight).normalize();
        if direction.is_nan() {
            Vec2::zero()
        } else {
            direction
        }
    }

    pub fn get_neighbours(&self, cell: &Cell) -> Vec<Cell> {
        let mut neighbours = Vec::new();
        if cell.x + 1 < self.width {
            neighbours.push((cell.x + 1, cell.y).into());
        }
        if cell.x > 0 {
            neighbours.push((cell.x - 1, cell.y).into());
        }
        if cell.y + 1 < self.height {
            neighbours.push((cell.x, cell.y + 1).into());
        }
        if cell.y > 0 {
            neighbours.push((cell.x, cell.y - 1).into());
        }
        neighbours
    }

    pub fn get_neighbours_cross(&self, cell: &Cell) -> Vec<Cell> {
        let mut neighbours = Vec::new();
        if cell.x + 1 < self.width && cell.y + 1 < self.height {
            if self.get(&(cell.x + 1, cell.y).into()) != std::u32::MAX
                && self.get(&(cell.x, cell.y + 1).into()) != std::u32::MAX
            {
                neighbours.push((cell.x + 1, cell.y + 1).into());
            }
        }
        if cell.x + 1 < self.width && cell.y > 0 {
            if self.get(&(cell.x + 1, cell.y).into()) != std::u32::MAX
                && self.get(&(cell.x, cell.y - 1).into()) != std::u32::MAX
            {
                neighbours.push((cell.x + 1, cell.y - 1).into());
            }
        }
        if cell.x > 0 && cell.y + 1 < self.height {
            if self.get(&(cell.x - 1, cell.y).into()) != std::u32::MAX
                && self.get(&(cell.x, cell.y + 1).into()) != std::u32::MAX
            {
                neighbours.push((cell.x - 1, cell.y + 1).into());
            }
        }
        if cell.x > 0 && cell.y > 0 {
            if self.get(&(cell.x - 1, cell.y).into()) != std::u32::MAX
                && self.get(&(cell.x, cell.y - 1).into()) != std::u32::MAX
            {
                neighbours.push((cell.x - 1, cell.y - 1).into());
            }
        }
        neighbours
    }

    fn position_to_cell(&self, position: &IVec2) -> Cell {
        (
            (position.x + (self.width / 2) as i32) as usize,
            (position.y + (self.height / 2) as i32) as usize,
        )
            .into()
    }

    fn mapped_position(&self, position: &Vec2) -> Vec2 {
        assert!(self.width % 2 == 0);
        assert!(self.height % 2 == 0);
        let x = position.x.floor() + (self.width / 2) as f32;
        let y = position.y.floor() + (self.height / 2) as f32;
        Vec2::new(x, y)
    }

    fn position_to_cell_floor(&self, position: &Vec2) -> Cell {
        let mapped_position = self.mapped_position(&position);
        (mapped_position.x as usize, mapped_position.y as usize).into()
    }

    fn map(&self, x: isize, y: isize) -> (usize, usize) {
        (
            (x + self.width as isize / 2) as usize,
            (y + self.height as isize / 2) as usize,
        )
    }

    pub fn set_destination(&mut self, destination: Vec2) {
        self.set_destination_i(
            destination.x.floor() as isize,
            destination.y.floor() as isize,
        );
    }

    pub fn set_destination_i(&mut self, x: isize, y: isize) {
        let (x, y) = self.map(x, y);
        self.set_destination_cell(Cell::new(x, y));
    }

    pub fn set_destination_cell(&mut self, cell: Cell) {
        let mut open = VecDeque::new();
        self.set(&cell, 0);
        open.push_back(cell);
        while !open.is_empty() {
            let cell = open.pop_front().unwrap();
            let value = self.get(&cell);
            for neighbour_cell in self.get_neighbours(&cell) {
                let n_value = self.get(&neighbour_cell);
                if n_value != std::u32::MAX && n_value > value + 100 {
                    self.set(&neighbour_cell, value + 100);
                    if !open.contains(&neighbour_cell) {
                        open.push_back(neighbour_cell);
                    }
                }
            }
            for neighbour_cell in self.get_neighbours_cross(&cell) {
                let n_value = self.get(&neighbour_cell);
                if n_value != std::u32::MAX && n_value > value + 141 {
                    self.set(&neighbour_cell, value + 141);
                    if !open.contains(&neighbour_cell) {
                        open.push_back(neighbour_cell);
                    }
                }
            }
        }
    }

    pub fn calculate_flow(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let current = Cell::new(x, y);
                let mut value = self.get(&current);
                let mut direction = IVec2::zero();
                if self.get(&current) != std::u32::MAX {
                    for neighbour in self.get_neighbours(&current) {
                        let n_value = self.get(&neighbour);
                        if n_value < value {
                            value = n_value;
                            direction = IVec2::new(
                                neighbour.x as i32 - current.x as i32,
                                neighbour.y as i32 - current.y as i32,
                            );
                        }
                    }
                    for neighbour in self.get_neighbours_cross(&current) {
                        let n_value = self.get(&neighbour);
                        if n_value < value {
                            value = n_value;
                            direction = IVec2::new(
                                neighbour.x as i32 - current.x as i32,
                                neighbour.y as i32 - current.y as i32,
                            );
                        }
                    }
                }
                self.set_flow_cell(current.x, current.y, direction);
            }
        }
    }

    fn print(&self) {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                print!("{:10} ", self.get(&(x, y).into()));
            }
            println!("");
        }
    }

    fn get_string_vector(v: &IVec2) -> String {
        let print_direction = match (v.x, v.y) {
            (-1, -1) => "⬋",
            (-1, 0) => "←",
            (-1, 1) => "⬉",
            (0, -1) => "↓",
            (0, 0) => " ",
            (0, 1) => "↑",
            (1, -1) => "⬊",
            (1, 0) => "→",
            (1, 1) => "⬈",
            _ => {
                assert!(false);
                " "
            }
        };
        print_direction.to_string()
    }

    pub fn print_flow(&self) {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let direction = self.get_flow_cell(x, y);
                print!(" {}", Self::get_string_vector(&direction));
            }
            println!("");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::navigation::{Cell, FlowField};
    use bevy::prelude::Vec2;

    #[test]
    fn get_neighbours_test() {
        let neighbours = FlowField::new(10, 10).get_neighbours(&(3, 3).into());
        assert_eq!(neighbours.len(), 4);
        assert!(neighbours.contains(&Cell::new(3, 4)));
        assert!(neighbours.contains(&Cell::new(3, 2)));
        assert!(neighbours.contains(&Cell::new(2, 3)));
        assert!(neighbours.contains(&Cell::new(4, 3)));
    }

    #[test]
    fn get_neighbours_test_edge() {
        let neighbours = FlowField::new(10, 10).get_neighbours(&(0, 0).into());
        assert_eq!(neighbours.len(), 2);
        assert!(neighbours.contains(&Cell::new(1, 0)));
        assert!(neighbours.contains(&Cell::new(0, 1)));
    }

    #[test]
    fn get_neighbours_cross_test() {
        let neighbours = FlowField::new(10, 10).get_neighbours_cross(&(3, 3).into());
        assert_eq!(neighbours.len(), 4);
        assert!(neighbours.contains(&Cell::new(4, 4)));
        assert!(neighbours.contains(&Cell::new(2, 4)));
        assert!(neighbours.contains(&Cell::new(4, 2)));
        assert!(neighbours.contains(&Cell::new(2, 2)));
    }

    #[test]
    fn get_neighbours_cross_edge_test() {
        let neighbours = FlowField::new(10, 10).get_neighbours_cross(&(9, 9).into());
        assert_eq!(neighbours.len(), 1);
        assert!(neighbours.contains(&Cell::new(8, 8)));
    }

    #[test]
    fn set_destination_test() {
        let mut flow_field = FlowField::new(10, 10);
        flow_field.set_destination_cell(Cell::new(4, 4));
        flow_field.print();
    }

    #[test]
    fn set_destination_with_one_blocked_test() {
        let mut flow_field = FlowField::new(10, 10);
        flow_field.set_blocked_cell(&Cell::new(3, 3));
        flow_field.set_destination_cell(Cell::new(4, 4));
        assert!(flow_field.get(&Cell::new(3, 3)) == std::u32::MAX);
        flow_field.print();
    }

    #[test]
    fn print_flow() {
        let mut flow_field = FlowField::new(25, 25);
        flow_field.set_destination_cell(Cell::new(10, 10));
        flow_field.calculate_flow();
        flow_field.print();
        flow_field.print_flow();
    }

    #[test]
    fn position_to_cell_test() {
        let flow_field = FlowField::new(4, 4);
        let cell = flow_field.position_to_cell_floor(&Vec2::new(0.5, 0.5));
        assert!(cell.x == 2 && cell.y == 2);
        let cell = flow_field.position_to_cell_floor(&Vec2::new(0.01, 0.01));
        assert!(cell.x == 2 && cell.y == 2);
        let cell = flow_field.position_to_cell_floor(&Vec2::new(-0.01, -0.01));
        assert!(cell.x == 1 && cell.y == 1);
        let cell = flow_field.position_to_cell_floor(&Vec2::new(-0.99, -0.99));
        assert!(cell.x == 1 && cell.y == 1);
        let cell = flow_field.position_to_cell_floor(&Vec2::new(-1.01, -1.01));
        assert!(cell.x == 0 && cell.y == 0);
    }

    #[test]
    fn interpolation_test_four_cells() {
        let mut flow_field = FlowField::new(4, 4);
        flow_field.set_destination(Vec2::new(0.5, 0.5));
        flow_field.calculate_flow();
        flow_field.print_flow();
        let flow = flow_field.get_flow_bilininterpol(&Vec2::new(0.5, -0.5));
    }
}
