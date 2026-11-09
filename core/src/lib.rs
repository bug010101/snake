use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::VecDeque;

/// 棋盘为 GRID x GRID 的方格。
pub const GRID: i32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub fn new(x: i32, y: i32) -> Self {
        Cell { x, y }
    }

    pub fn add(self, d: (i32, i32)) -> Cell {
        Cell {
            x: self.x + d.0,
            y: self.y + d.1,
        }
    }
}

/// 贪吃蛇游戏状态。核心逻辑全部为纯函数式 step，便于单元测试覆盖边界。
pub struct Game {
    pub snake: Vec<Cell>,
    pub dir: Direction,
    /// 输入队列：连续快速转向时按序消费，避免“快速反向自杀”这类经典 bug。
    pub queue: VecDeque<Direction>,
    pub food: Cell,
    pub score: u32,
    pub alive: bool,
    pub won: bool,
    rng: SmallRng,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mid = GRID / 2;
        let mut g = Game {
            snake: vec![
                Cell::new(mid, mid),
                Cell::new(mid - 1, mid),
                Cell::new(mid - 2, mid),
            ],
            dir: Direction::Right,
            queue: VecDeque::new(),
            food: Cell::new(-1, -1),
            score: 0,
            alive: true,
            won: false,
            rng,
        };
        g.food = g.spawn_food();
        g
    }

    /// 登记一个转向意图。相对“最近一次意图方向（或当前方向）”做校验，
    /// 既禁止 180° 反向，又允许快速连续转向（如 右 -> 上 -> 左）。
    pub fn set_direction(&mut self, d: Direction) {
        if !self.alive {
            return;
        }
        let last = self.queue.back().copied().unwrap_or(self.dir);
        if d != last.opposite() && d != last {
            if self.queue.len() < 3 {
                self.queue.push_back(d);
            }
        }
    }

    fn spawn_food(&mut self) -> Cell {
        let empty: Vec<Cell> = (0..GRID)
            .flat_map(|y| (0..GRID).map(move |x| Cell::new(x, y)))
            .filter(|c| !self.snake.contains(c))
            .collect();
        if empty.is_empty() {
            // 棋盘填满 = 胜利
            self.won = true;
            self.alive = false;
            return Cell::new(-1, -1);
        }
        let i = self.rng.gen_range(0..empty.len());
        empty[i]
    }

    /// 推进一格。碰撞/越界判定均基于“移动后”的蛇身，正确处理“尾巴让位”的情况。
    pub fn step(&mut self) {
        if !self.alive {
            return;
        }
        if let Some(d) = self.queue.pop_front() {
            self.dir = d;
        }

        let new_head = self.snake[0].add(self.dir.delta());

        // 撞墙
        if new_head.x < 0 || new_head.x >= GRID || new_head.y < 0 || new_head.y >= GRID {
            self.alive = false;
            return;
        }

        let will_eat = new_head == self.food;

        // 构建新蛇身：吃则保留尾巴（增长），否则去掉尾巴（移动）
        let mut new_snake = Vec::with_capacity(self.snake.len() + 1);
        new_snake.push(new_head);
        if will_eat {
            new_snake.extend(self.snake.iter().copied());
        } else {
            new_snake.extend(self.snake[..self.snake.len() - 1].iter().copied());
        }

        // 撞自己（新头落在身体上）
        if new_snake[1..].contains(&new_head) {
            self.alive = false;
            return;
        }

        self.snake = new_snake;

        if will_eat {
            self.score += 1;
            self.food = self.spawn_food();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_ok() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn step_moves_and_keeps_length() {
        let mut g = Game::new(42);
        // 固定食物到安全格，避免随机食物恰好落在前进方向导致“误判增长”
        g.food = Cell::new(0, 0);
        let head = g.snake[0];
        g.step();
        assert_eq!(g.snake[0], head.add(Direction::Right.delta()));
        assert_eq!(g.snake.len(), 3);
        assert!(g.alive);
    }

    #[test]
    fn reversal_blocked() {
        let mut g = Game::new(1);
        g.set_direction(Direction::Left); // 相对 Right 是反向 -> 拒绝
        assert!(g.queue.is_empty());
        g.set_direction(Direction::Up); // 允许
        assert_eq!(g.queue.back(), Some(&Direction::Up));
        g.set_direction(Direction::Down); // 相对待执行 Up 是反向 -> 拒绝
        assert_eq!(g.queue.back(), Some(&Direction::Up));
    }

    #[test]
    fn queue_allows_turn_then_perp() {
        let mut g = Game::new(2);
        g.food = Cell::new(0, 0); // 固定食物，排除随机食物对转向流程的干扰
        g.set_direction(Direction::Up); // [Up]
        g.set_direction(Direction::Left); // [Up, Left]，Left 相对 Up 合法
        assert_eq!(g.queue.len(), 2);
        g.step(); // dir = Up
        assert_eq!(g.dir, Direction::Up);
        g.step(); // dir = Left
        assert_eq!(g.dir, Direction::Left);
        assert!(g.alive);
    }

    #[test]
    fn eats_food_grows_scores() {
        let mut g = Game::new(7);
        let head = g.snake[0];
        g.food = head.add(Direction::Right.delta());
        let len = g.snake.len();
        g.step();
        assert_eq!(g.snake.len(), len + 1);
        assert_eq!(g.score, 1);
        assert!(g.food.x >= 0 && g.food.x < GRID && g.food.y >= 0 && g.food.y < GRID);
    }

    #[test]
    fn wall_kills() {
        let mut g = Game::new(3);
        g.snake = vec![Cell::new(0, 5), Cell::new(1, 5), Cell::new(2, 5)];
        g.dir = Direction::Left;
        g.queue.clear();
        g.step();
        assert!(!g.alive);
    }

    #[test]
    fn self_collision_kills() {
        let mut g = Game::new(5);
        g.snake = vec![
            Cell::new(5, 5),
            Cell::new(4, 5),
            Cell::new(4, 6),
            Cell::new(5, 6),
            Cell::new(5, 7),
        ];
        g.dir = Direction::Down; // 头 (5,5) 下移 -> (5,6) 撞到非尾身体
        g.queue.clear();
        g.food = Cell::new(0, 0);
        g.step();
        assert!(!g.alive);
    }
}
