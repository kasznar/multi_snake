use serde::{Deserialize, Serialize};

const SNAKE_DEFAULT_LENGTH: u32 = 5;

#[derive(Serialize, PartialEq)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Copy, Clone, Serialize)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize)]
pub struct Snake {
    pub body: Vec<Point>,
    pub direction: Direction,
}

impl Snake {
    pub fn new(mut head: Point) -> Snake {
        if head.x < SNAKE_DEFAULT_LENGTH {
            head = Point{x: SNAKE_DEFAULT_LENGTH, y: head.y};
        }

        let mut body: Vec<Point> = vec![head];

        for i in 1..SNAKE_DEFAULT_LENGTH {
            body.push(Point{x: head.x - i, y: head.y})
        }

        Snake{
            direction: Direction::RIGHT,
            body,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) {
        let valid = match direction {
            Direction::RIGHT => self.direction != Direction::LEFT,
            Direction::LEFT => self.direction != Direction::RIGHT,
            Direction::UP => self.direction != Direction::DOWN,
            Direction::DOWN => self.direction != Direction::UP,
        };

        if valid {
            self.direction = direction;
        }
    }

    pub fn update(&mut self) {
        let head = &self.body[0];

        let new_head_result = match self.direction {
            Direction::RIGHT => Some(Point{x: head.x + 1, y: head.y}),
            Direction::LEFT => if head.x > 0 {Some(Point{x: head.x - 1, y: head.y})} else {None},
            Direction::UP => if head.y > 0 {Some(Point{x: head.x, y: head.y - 1})} else {None},
            Direction::DOWN => Some(Point{x: head.x, y: head.y + 1}),
        };

        if let Some(new_head) = new_head_result {
            self.body.insert(0, new_head);
            self.body.pop();
        }

    }
}