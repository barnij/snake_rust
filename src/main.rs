use ggez;
use rand;

use ggez::event::{KeyCode, KeyMods};
use ggez::{ event,
            graphics::{self, DrawParam},
            Context,
            GameResult};

use std::collections::LinkedList;
use std::time::{Duration, Instant};
use std::env;
use std::path;
use ggez::mint::Point2;


use rand::Rng;

const GRID_SIZE: (i16, i16) = (30, 20);
const GRID_CELL_SIZE: (i16, i16) = (40, 40);

const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

const UPDATES_PER_SECOND: f32 = 10.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i16,
    y: i16,
}


trait ModuloSigned {
    fn modulo(&self, n: Self) -> Self;
}

impl<T> ModuloSigned for T
where
    T: std::ops::Add<Output = T> + std::ops::Rem<Output = T> + Clone,
{
    fn modulo(&self, n: T) -> T {
        (self.clone() % n.clone() + n.clone()) % n.clone()
    }
}

impl GridPosition {

    pub fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }

    pub fn random(min_x: i16, min_y: i16, max_x: i16, max_y: i16) -> Self {
        let mut rng = rand::thread_rng();
        (
            rng.gen_range::<i16, i16, i16>(min_x, max_x),
            rng.gen_range::<i16, i16, i16>(min_y, max_y),
        )
        .into()
    }

    pub fn new_from_move(pos: GridPosition, dir: Direction) -> Self {
        match dir {
            Direction::Up => GridPosition::new(pos.x, (pos.y - 1).modulo(GRID_SIZE.1)),
            Direction::Down => GridPosition::new(pos.x, (pos.y + 1).modulo(GRID_SIZE.1)),
            Direction::Left => GridPosition::new((pos.x - 1).modulo(GRID_SIZE.0), pos.y),
            Direction::Right => GridPosition::new((pos.x + 1).modulo(GRID_SIZE.0), pos.y),
            Direction::None => GridPosition::new(pos.x, pos.y),
        }
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_CELL_SIZE.0 as i32,
            pos.y as i32 * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

impl From<GridPosition> for Point2<f32> {
    fn from(pos: GridPosition) -> Self {
        Point2 {
            x: pos.x as f32 * GRID_CELL_SIZE.0 as f32,
            y: pos.y as f32 * GRID_CELL_SIZE.1 as f32,
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {

    pub fn inverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            _ => Direction::None,
        }
    }

    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Segment {
    pos: GridPosition,
    dir: Direction,
}

impl Segment {
    pub fn new(pos: GridPosition, dir: Direction) -> Self {
        Segment { pos, dir }
    }
}

struct Food {
    pos: GridPosition,
    image: graphics::Image,
}

impl Food {
    pub fn new(pos: GridPosition, ctx: &mut Context) -> GameResult<Food> {
        let image = graphics::Image::new(ctx, "/mouse.png")?;
        let s = Food { pos, image };
        Ok(s)
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let pnt2: Point2<f32> = self.pos.into();
        graphics::draw(ctx, &self.image, (pnt2,))?;
        Ok(())
    }
}

struct Wall {
    list: LinkedList<Segment>,
    image: graphics::Image,
}

impl Wall {
    pub fn new(ctx: &mut Context) -> GameResult<Wall> {
        let mut list = LinkedList::new();
        for i in 0..GRID_SIZE.0{
            for j in 0..GRID_SIZE.1{
                if i == 0 || j==0 || i+1 == GRID_SIZE.0 || j+1 == GRID_SIZE.1{
                    list.push_back(Segment::new((i as i16, j as i16).into(), Direction::None));
                }
            }
        }

        let image = graphics::Image::new(ctx, "/wall.jpg")?;

        let s = Wall {
            list,
            image: image,
        };

        Ok(s)
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for seg in self.list.iter() {
            let pnt2: Point2<f32> = seg.pos.into();
            graphics::draw(ctx, &self.image, (pnt2,))?;
        }
        Ok(())
    }
}


#[derive(Clone, Copy, Debug)]
enum Ate {
    Itself,
    Food,
    Wall,
}


struct Snake {

    head: Segment,
    dir: Direction,
    body: LinkedList<Segment>,
    tail: Segment,
    ate: Option<Ate>,
    last_update_dir: Direction,
    next_dir: Option<Direction>,
    head_image: graphics::Image,
    body_image: graphics::Image,
    turn_body_image: graphics::Image,
    tail_image: graphics::Image,
}

impl Snake {
    pub fn new(pos: GridPosition, ctx: &mut Context) -> GameResult<Snake> {
        let body = LinkedList::new();
        let head_image = graphics::Image::new(ctx, "/shead.png")?;
        let body_image = graphics::Image::new(ctx, "/sbody.png")?;
        let turn_body_image = graphics::Image::new(ctx, "/sturn.png")?;
        let tail_image = graphics::Image::new(ctx, "/send.png")?;

        let s = Snake {
            head: Segment::new(pos, Direction::Right),
            dir: Direction::Right,
            last_update_dir: Direction::Right,
            body: body,
            tail: Segment::new((pos.x - 1, pos.y).into(), Direction::Right),
            ate: None,
            next_dir: None,
            head_image: head_image,
            body_image: body_image,
            turn_body_image: turn_body_image,
            tail_image: tail_image,
        };
        Ok(s)
    }

    fn eats(&self, food: &Food) -> bool {
        if self.head.pos == food.pos {
            true
        } else {
            false
        }
    }

    fn eats_self(&self) -> bool {
        for seg in self.body.iter() {
            if self.head.pos == seg.pos {
                return true;
            }
        }
        false
    }

    fn collides(&self, walls: &Wall) -> bool {
        for wall in walls.list.iter() {
            if self.head.pos == wall.pos {
                return true;
            }
        }
        false
    }


    fn update(&mut self, food: &Food, walls: &Wall) {

        if self.last_update_dir == self.dir && self.next_dir.is_some() {
            self.dir = self.next_dir.unwrap();
            self.next_dir = None;
        }

        let new_head_pos = GridPosition::new_from_move(self.head.pos, self.dir);
        let new_head = Segment::new(new_head_pos, self.dir);
        self.body.push_back(self.head.clone());
        self.head = new_head;
        if self.eats_self() {
            self.ate = Some(Ate::Itself);
        } else if self.eats(food) {
            self.ate = Some(Ate::Food);
        } else if self.collides(walls){
            self.ate = Some(Ate::Wall);
        } else {
            self.ate = None;
        }

        if let None = self.ate {
            self.tail = self.body.front().unwrap().clone();
            self.body.pop_front();
            if self.body.is_empty(){
                self.tail.dir = self.head.dir;
            }else{
                self.tail.dir = self.body.front().unwrap().dir;
            }
        }

        self.last_update_dir = self.dir;
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {

        let drawparam = DrawParam::default();

        let mut iter = self.body.iter();
        iter.next();
        for seg in self.body.iter() {
            let next_seg = iter.next();
            let dir = next_seg.unwrap_or(&self.head).dir;
            let pnt2: Point2<f32> = seg.pos.into();

            if dir != seg.dir{
                let param = get_param_for_turned(seg.dir, dir);
                graphics::draw(ctx, &self.turn_body_image, drawparam.rotation(param.0).offset(param.1).dest(pnt2))?;
            }else{
                let param = get_param(dir);
                graphics::draw(ctx, &self.body_image, drawparam.rotation(param.0).offset(param.1).dest(pnt2))?;
            }
        }

        let mut param = get_param(self.dir);
        let mut pnt2: Point2<f32> = self.head.pos.into();
        graphics::draw(ctx, &self.head_image, drawparam.rotation(param.0).offset(param.1).dest(pnt2))?;

        pnt2 = self.tail.pos.into();
        param = get_param(self.tail.dir);
        graphics::draw(ctx, &self.tail_image, drawparam.rotation(param.0).offset(param.1).dest(pnt2))?;

        Ok(())
    }
}

fn get_param(dir: Direction) -> (f32, Point2<f32>){
        let mut offset = Point2 {x:0.0, y:0.0};
        let mut rotation = 0.0;
        match dir {
            Direction::Down => {
                offset = Point2 {x:0.98, y:0.98};
                rotation = std::f32::consts::PI;
            },
            Direction::Right => {
                offset = Point2 {x:0.0, y:1.0};
                rotation = std::f32::consts::PI/2.0;
            },
            Direction::Left => {
                offset = Point2 {x:1.0, y:0.0};
                rotation = -std::f32::consts::PI/2.0;
            }
            _ => {},
        }
        (rotation, offset)
}

fn get_param_for_turned(mydir: Direction, nextdir: Direction) -> (f32, Point2<f32>){
        let mut offset = Point2 {x:0.0, y:0.0};
        let mut rotation = 0.0;
        match nextdir {
            Direction::Up => {
                if mydir == Direction::Right{
                    offset = Point2 {x:0.98, y:0.98};
                    rotation = std::f32::consts::PI;
                }else if mydir == Direction::Left {
                    offset = Point2 {x:1.0, y:0.0};
                    rotation = -std::f32::consts::PI/2.0;
                }
            },
            Direction::Right => {
                if mydir == Direction::Down{
                    offset = Point2 {x:1.0, y:0.0};
                    rotation = -std::f32::consts::PI/2.0;
                }
            },
            Direction::Left => {
                if mydir == Direction::Down{
                    offset = Point2 {x:0.98, y:0.98};
                    rotation = std::f32::consts::PI;
                }else if mydir == Direction::Up{
                    offset = Point2 {x:0.0, y:1.0};
                    rotation = std::f32::consts::PI/2.0;
                }
            }
            Direction::Down => {
                if mydir == Direction::Right {
                    offset = Point2 {x: 0.0, y:1.0};
                    rotation = std::f32::consts::PI/2.0;
                } else if mydir == Direction::Left {
                    offset = Point2 {x: -0.02, y:0.0};
                }
            }
            _ => {},
        }
        (rotation, offset)
}


struct GameState {
    snake: Snake,
    food: Food,
    walls: Wall,
    gameover: bool,
    last_update: Instant,
    floor_image: graphics::Image,
}

impl GameState {

    pub fn new(ctx: &mut Context) -> GameResult<GameState> {

        let snake_pos = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        let food_pos = GridPosition::random(1, 1, GRID_SIZE.0 - 1, GRID_SIZE.1 - 1);
        let image = graphics::Image::new(ctx, "/floor.png")?;


        let s = GameState {
            snake: Snake::new(snake_pos, ctx).unwrap(),
            food: Food::new(food_pos, ctx).unwrap(),
            walls: Wall::new(ctx).unwrap(),
            gameover: false,
            last_update: Instant::now(),
            floor_image: image,
        };

        Ok(s)
    }

    fn draw_floor(&mut self, ctx: &mut Context) -> GameResult {
        for i in 0..GRID_SIZE.0{
            for j in 0..GRID_SIZE.1{
                if i > 0 || j > 0 || i+1 < GRID_SIZE.0 || j+1 < GRID_SIZE.1{
                    let gp: GridPosition = (i as i16, j as i16).into();
                    let pnt2: ggez::mint::Point2<f32> = gp.into();
                    graphics::draw(ctx, &self.floor_image, (pnt2,))?;
                }
            }
        }
        Ok(())
    }
}

impl event::EventHandler for GameState {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            if !self.gameover {

                self.snake.update(&self.food, &self.walls);

                if let Some(ate) = self.snake.ate {
                    match ate {
                        Ate::Food => {
                            let new_food_pos = GridPosition::random(1, 1, GRID_SIZE.0 - 1, GRID_SIZE.1 - 1);
                            self.food.pos = new_food_pos;
                        }
                        Ate::Itself | Ate::Wall => {
                            self.gameover = true;
                        }
                    }
                }
            }

            self.last_update = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
        //self.draw_floor(ctx)?;
        self.snake.draw(ctx)?;
        self.food.draw(ctx)?;
        self.walls.draw(ctx)?;
        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {

        if let Some(dir) = Direction::from_keycode(keycode) {

            if self.snake.dir != self.snake.last_update_dir && dir.inverse() != self.snake.dir {
                self.snake.next_dir = Some(dir);
            } else if dir.inverse() != self.snake.last_update_dir {
                self.snake.dir = dir;
            }
        }
    }
}

fn main() -> GameResult {

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("Snake in Rust", "Bartosz Jaśkiewicz")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Snake"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = &mut GameState::new(ctx).unwrap();

    event::run(ctx, events_loop, state)
}