mod flat2darr;

use std::io::stdout;

use crossterm::{ style::Print, terminal::ClearType, ExecutableCommand };
use flat2darr::Flat2DArray;
use rand::prelude::SliceRandom;
use rand::thread_rng;

pub const WIDTH: usize = 100;
pub const HEIGHT: usize = 100;
const MAX_DEPTH: usize = 2000;

const UP: (isize, isize) = (0, -1);
const DOWN: (isize, isize) = (0, 1);
const LEFT: (isize, isize) = (-1, 0);
const RIGHT: (isize, isize) = (1, 0);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Cell {
    #[default]
    Empty,
    Wall((isize, isize)), // tuple is direction
    Path(bool, (isize, isize)), // true if winning path (tuple is direction)
    Start,
    End,
    Invalid,
}

impl Cell {
    fn is_wall(&self) -> bool {
        matches!(self, Cell::Wall(_))
    }

    fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    fn is_some(&self) -> bool {
        !self.is_empty()
    }

    fn is_winning_path(&self) -> bool {
        matches!(self, Cell::Path(true, _))
    }
}

#[derive(Clone)]
struct Maze {
    width: usize,
    height: usize,
    cells: Flat2DArray<Cell>,
    visited: Flat2DArray<bool>,
    rng: rand::rngs::ThreadRng,
}

impl Maze {
    fn new(width: usize, height: usize) -> Maze {
        Maze {
            width,
            height,
            cells: Flat2DArray::new(),
            visited: Flat2DArray::new(),
            rng: thread_rng(),
        }
    }

    fn get(&self, x: usize, y: usize) -> Cell {
        self.cells.get(y, x).copied().unwrap_or(Cell::Invalid)
    }

    fn set(&mut self, x: usize, y: usize, value: Cell) {
        self.cells.set(y, x, value);
    }

    fn generate_winning_path(
        &mut self,
        x: usize,
        y: usize,
        prev_direction: Option<(isize, isize)>,
        depth: usize
    ) -> bool {
        if depth > MAX_DEPTH {
            return false;
        }

        if x == self.width - 1 && y == self.height - 1 {
            self.set(x, y, Cell::End);
            return true;
        }

        self.visited.set(y, x, true);

        let mut directions = vec![UP, DOWN, LEFT, RIGHT];

        directions.shuffle(&mut self.rng);

        // Bias towards continuing in the same direction
        if rand::random::<f32>() < 0.8 && prev_direction.is_some() {
            directions = vec![prev_direction.unwrap()];
        }

        for (dx, dy) in directions {
            let nx = (x as isize) + dx;
            let ny = (y as isize) + dy;
            if nx < 0 || ny < 0 || nx >= (self.width as isize) || ny >= (self.height as isize) {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if self.visited.get(ny, nx).copied().unwrap() || self.get(nx, ny).is_winning_path() {
                continue;
            }
            // if self.count_surrounding_walls(nx, ny, true) > 1 {
            //     continue;
            // }
            self.set(x, y, if x == 0 && y == 0 { Cell::Start } else { Cell::Path(true, (dx, dy)) });
            self.print(true, depth);
            if self.generate_winning_path(nx, ny, Some((dx, dy)), depth + 1) {
                return true; // Return true if the path leads to a solution
            }
            // Backtrack
            self.set(nx, ny, Cell::Empty);
            // self.visited.set(ny, nx, false);
        }

        false // Return false if no path is found
    }

    fn generate_other_paths(
        &mut self,
        x: usize,
        y: usize,
        prev_direction: Option<(isize, isize)>,
        depth: usize,
        max_depth: usize
    ) {
        if depth > MAX_DEPTH || depth > max_depth {
            return;
        }

        self.visited.set(y, x, true);

        let mut directions = vec![(0, -1), (0, 1), (-1, 0), (1, 0)];

        directions.shuffle(&mut self.rng);

        // Bias towards continuing in the same direction
        if rand::random::<f32>() < 0.5 && prev_direction.is_some() {
            directions = vec![prev_direction.unwrap()];
        }

        for (dx, dy) in directions {
            let nx = (x as isize) + dx;
            let ny = (y as isize) + dy;
            if nx < 0 || ny < 0 || nx >= (self.width as isize) || ny >= (self.height as isize) {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if self.count_surrounding_walls(nx, ny, false) > 1 {
                continue;
            }
            if self.get(nx, ny).is_empty() {
                self.set(x, y, Cell::Path(false, (dx, dy)));
            }
            // self.print(true, depth);
            self.generate_other_paths(nx, ny, Some((dx, dy)), depth + 1, max_depth);
        }
    }

    fn generate_walls(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                match self.get(x, y) {
                    Cell::Path(_, dir) => {
                        // get all 4 cells around the current cell
                        let mut cells = vec![];

                        let left = self.get(((x as isize) - 1) as usize, y);
                        let right = self.get(((x as isize) + 1) as usize, y);
                        let top = self.get(x, ((y as isize) - 1) as usize);
                        let bottom = self.get(x, ((y as isize) + 1) as usize);

                        if left.is_some() {
                            cells.push(left);
                        }
                        if right.is_some() {
                            cells.push(right);
                        }
                        if top.is_some() {
                            cells.push(top);
                        }
                        if bottom.is_some() {
                            cells.push(bottom);
                        }

                        for cell in cells {
                            if !cell.is_winning_path() {
                                self.set(x, y, Cell::Wall(dir));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // get tuple of left and right cells or top and bottom cells
    // fn get_adjacent(&self, x: usize, y: usize, dir: (isize, isize)) -> Option<(Cell, Cell)> {
    //     let cell1 = self.get(((x as isize) + dir.1) as usize, ((y as isize) - dir.0) as usize);
    //     let cell2 = self.get(((x as isize) - dir.1) as usize, ((y as isize) + dir.0) as usize);
    //     if cell1.is_some() && cell2.is_some() {
    //         Some((cell1, cell2))
    //     } else {
    //         None
    //     }
    // }

    fn count_surrounding_walls(&self, x: usize, y: usize, count_winning_path: bool) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = (x as isize) + dx;
                let ny = (y as isize) + dy;
                if nx < 0 || ny < 0 || nx >= (self.width as isize) || ny >= (self.height as isize) {
                    continue;
                } else if
                    self.get(nx as usize, ny as usize).is_some() &&
                    (count_winning_path || !self.get(nx as usize, ny as usize).is_winning_path())
                {
                    count += 1;
                }
            }
        }
        count
    }

    fn print(&self, use_crossterm: bool, depth: usize) {
        if use_crossterm {
            // clear the terminal
            stdout().execute(crossterm::terminal::Clear(ClearType::All)).unwrap();
            stdout().execute(crossterm::cursor::MoveTo(0, 0)).unwrap();
            for y in 0..self.height {
                for x in 0..self.width {
                    match self.get(x, y) {
                        Cell::Empty => print!(" "),
                        Cell::Wall(_) => print!("#"),
                        Cell::Path(e, _) => print!("{}", if e { "." } else { "," }),
                        Cell::Start => print!("S"),
                        Cell::End => print!("E"),
                        Cell::Invalid => print!("X"),
                    }
                }
                println!();
            }
            stdout().execute(crossterm::cursor::MoveTo(10, 0)).unwrap();
            stdout()
                .execute(Print(format!("Depth: {}", depth)))
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
        } else {
            for y in 0..self.height {
                for x in 0..self.width {
                    match self.get(x, y) {
                        Cell::Empty => print!(" "),
                        Cell::Wall(dir) => {
                            match dir {
                                UP | DOWN => print!("█"),
                                LEFT | RIGHT => print!("▬"),
                                _ => print!("?"),
                            }
                        }
                        Cell::Path(e, _) => print!("{}", if e { "." } else { "," }),
                        Cell::Start => print!("S"),
                        Cell::End => print!("E"),
                        Cell::Invalid => print!("X"),
                    }
                }
                println!();
            }
        }
    }

    fn to_image(&self, path: &str, show_win_path: bool) {
        let mut imgbuf = image::ImageBuffer::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = match self.get(x, y) {
                    Cell::Empty => image::Rgb([0u8, 0u8, 0u8]),
                    Cell::Wall(_) => image::Rgb([255u8, 255u8, 255u8]),
                    Cell::Path(_, _) => {
                        if show_win_path {
                            image::Rgb([255u8, 0u8, 0u8])
                        } else {
                            image::Rgb([0u8, 0u8, 0u8])
                        }
                    }
                    Cell::Start => image::Rgb([0u8, 255u8, 0u8]),
                    Cell::End => image::Rgb([0u8, 0u8, 255u8]),
                    Cell::Invalid => image::Rgb([255u8, 255u8, 0u8]),
                };
                imgbuf.put_pixel(x as u32, y as u32, pixel);
            }
        }
        imgbuf.save(path).unwrap();
    }
}

fn main() {
    // enter alternate screen mode
    // stdout().execute(crossterm::terminal::EnterAlternateScreen).unwrap();
    let mut maze = Maze::new(WIDTH, HEIGHT);
    let mut retry = 0;

    for _ in 0..WIDTH * HEIGHT {
        maze.generate_other_paths(
            rand::random::<usize>() % WIDTH,
            rand::random::<usize>() % HEIGHT,
            None,
            0,
            5
        );
    }

    maze.generate_walls();
    let maze_copy = maze.clone();
    while !maze.generate_winning_path(0, 0, None, 0) {
        maze = maze_copy.clone();
        retry += 1;
        // println!("retry: {}", retry);
    }
    // maze.print(false, 0);
    maze.to_image("maze.png", false);
    maze.to_image("maze_win_path.png", true);
    println!("retry: {}", retry);
}
