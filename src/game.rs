use serde::{Serialize};

const SNAKE_DEFAULT_LENGTH: u32 = 5;

struct Resolution {
    width: u32,
    height: u32,
}

const GAME_RESOLUTION: Resolution = Resolution {
    width: 80,
    height: 60,
};

#[derive(Copy, Clone, Serialize, PartialEq)]
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

#[derive(Serialize, Clone)]
pub struct Snake {
    pub body: Vec<Point>,
    pub direction: Direction,
}

impl Snake {
    pub fn new(head: Point, direction: Direction) -> Snake {
        let shifted_head = Self::shift_head(head, direction);
        let body = Self::create_body(shifted_head, direction);


        Snake {
            direction,
            body,
        }
    }

    fn shift_head(head: Point, direction: Direction) -> Point {
        let mut shifted_head = head;

        match direction {
            Direction::RIGHT => {
                if head.x < SNAKE_DEFAULT_LENGTH {
                    shifted_head = Point { x: SNAKE_DEFAULT_LENGTH, y: head.y };
                }
            }
            Direction::LEFT => {
                let max_x = GAME_RESOLUTION.width - SNAKE_DEFAULT_LENGTH;

                if head.x > max_x {
                    shifted_head = Point { x: max_x, y: head.y };
                }
            }
            Direction::UP => {
                let max_y = GAME_RESOLUTION.height - SNAKE_DEFAULT_LENGTH;

                if head.y > max_y {
                    shifted_head = Point { x: head.x, y: max_y };
                }
            }
            Direction::DOWN => {
                if head.y < SNAKE_DEFAULT_LENGTH {
                    shifted_head = Point { x: head.x, y: SNAKE_DEFAULT_LENGTH };
                }
            }
        }

        shifted_head
    }

    fn create_body(head: Point, direction: Direction) -> Vec<Point> {
        let mut body: Vec<Point> = vec![head];

        match direction {
            Direction::RIGHT => {
                for i in 1..SNAKE_DEFAULT_LENGTH {
                    body.push(Point { x: head.x - i, y: head.y })
                }
            }
            Direction::LEFT => {
                for i in 1..SNAKE_DEFAULT_LENGTH {
                    body.push(Point { x: head.x + i, y: head.y })
                }
            }
            Direction::UP => {
                for i in 1..SNAKE_DEFAULT_LENGTH {
                    body.push(Point { x: head.x, y: head.y - i })
                }
            }
            Direction::DOWN => {
                for i in 1..SNAKE_DEFAULT_LENGTH {
                    body.push(Point { x: head.x, y: head.y + i })
                }
            }
        }

        body
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
            Direction::RIGHT => Some(Point { x: head.x + 1, y: head.y }),
            Direction::LEFT => if head.x > 0 { Some(Point { x: head.x - 1, y: head.y }) } else { None },
            Direction::UP => if head.y > 0 { Some(Point { x: head.x, y: head.y - 1 }) } else { None },
            Direction::DOWN => Some(Point { x: head.x, y: head.y + 1 }),
        };

        if let Some(new_head) = new_head_result {
            if new_head.x > GAME_RESOLUTION.width - 1 || new_head.y > GAME_RESOLUTION.height - 1 {
                return;
            }

            self.body.insert(0, new_head);
            self.body.pop();
        }
    }
}

#[derive(Copy, Clone, Serialize)]
pub enum Player {
    Player1,
    Player2,
}

#[derive(Serialize, Clone)]
pub struct Game {
    players: (Snake, Snake),
    winner: Option<Player>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: (
                Snake::new(Point { x: 0, y: 28 }, Direction::RIGHT),
                Snake::new(Point { x: 80, y: 29 }, Direction::LEFT),
            ),
            winner: None,
        }
    }

    pub fn tick(&mut self) {
        self.players.0.update();
        self.players.1.update();
    }

    pub fn change_direction(&mut self, player: Player, direction: Direction) {
        match player {
            Player::Player1 => {
                self.players.0.set_direction(direction);
            }
            Player::Player2 => {
                self.players.1.set_direction(direction);
            }
        }
    }
}