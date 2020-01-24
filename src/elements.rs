use std::collections::LinkedList;

use ggez::event::{KeyCode};
use ggez::mint::Point2;
use ggez::{graphics::{self, DrawParam},
           Context,
           GameResult};

use rand;
use rand::Rng;

use crate::consts::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GridPosition {
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
pub enum Direction {
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
pub struct Segment {
    pub pos: GridPosition,
    dir: Direction,
}

impl Segment {
    pub fn new(pos: GridPosition, dir: Direction) -> Self {
        Segment { pos, dir }
    }
}

pub struct Food {
    pub pos: GridPosition,
    image: graphics::Image,
}

impl Food {
    pub fn new(pos: GridPosition, ctx: &mut Context) -> GameResult<Food> {
        let image = graphics::Image::new(ctx, "/mouse.png")?;
        let s = Food { pos, image };
        Ok(s)
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult {
        let pnt2: Point2<f32> = self.pos.into();
        graphics::draw(ctx, &self.image, (pnt2,))?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Wall {
    list: LinkedList<Segment>,
    image: graphics::Image,
}

fn if_hole() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>() % 100 < 30
}

impl Wall {
    pub fn new(ctx: &mut Context) -> GameResult<Wall> {
        let mut list = LinkedList::new();
        for i in 0..GRID_SIZE.0{
            for j in 0..GRID_SIZE.1{
                if j==1 || j+1 == GRID_SIZE.1 {
                    list.push_back(Segment::new((i as i16, j as i16).into(), Direction::None));
                } else if (i == 0 || i+1 == GRID_SIZE.0) && !if_hole() && j != 0 {
                    list.push_back(Segment::new((i as i16, j as i16).into(), Direction::None));
                }
            }
        }

        let image = graphics::Image::new(ctx, "/wall.png")?;

        let s = Wall {
            list,
            image: image,
        };

        Ok(s)
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for seg in self.list.iter() {
            let pnt2: Point2<f32> = seg.pos.into();
            graphics::draw(ctx, &self.image, (pnt2,))?;
        }
        Ok(())
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Ate {
    Itself,
    Food,
    Wall,
}

#[derive(Clone, Debug)]
pub struct Snake {

    head: Segment,
    pub dir: Direction,
    body: LinkedList<Segment>,
    tail: Segment,
    pub ate: Option<Ate>,
    pub last_update_dir: Direction,
    pub next_dir: Option<Direction>,
    head_image: graphics::Image,
    body_image: graphics::Image,
    turn_body_image: graphics::Image,
    tail_image: graphics::Image,
    blood_image: graphics::Image,
    blood_wall_image: graphics::Image,
}

impl Snake {
    pub fn new(pos: GridPosition, ctx: &mut Context) -> GameResult<Snake> {
        let body = LinkedList::new();
        let head_image = graphics::Image::new(ctx, "/shead.png")?;
        let body_image = graphics::Image::new(ctx, "/sbody.png")?;
        let turn_body_image = graphics::Image::new(ctx, "/sturn.png")?;
        let tail_image = graphics::Image::new(ctx, "/send.png")?;
        let blood_image = graphics::Image::new(ctx, "/blood.png")?;
        let blood_wall_image = graphics::Image::new(ctx, "/holewall.png")?;

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
            blood_image: blood_image,
            blood_wall_image: blood_wall_image,
        };
        Ok(s)
    }

    pub fn eats(&self, food: &Food) -> bool {
        if self.head.pos == food.pos {
            true
        } else {
            false
        }
    }

    pub fn eats_self(&self) -> bool {
        for seg in self.body.iter() {
            if self.head.pos == seg.pos {
                return true;
            }
        }
        false
    }

    pub fn collides(&self, walls: &Wall) -> bool {
        for wall in walls.list.iter() {
            if self.head.pos == wall.pos {
                return true;
            }
        }
        false
    }


    pub fn update(&mut self, food: &Food, walls: &Wall) {

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

    pub fn draw(&self, ctx: &mut Context, gameover: bool) -> GameResult<()> {

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

        if gameover{
            pnt2 = self.head.pos.into();
            let x = self.head.pos.x;
            let y = self.head.pos.y;
            if x == 0 || x+1 == GRID_SIZE.0 || y == 1 || y+1 == GRID_SIZE.1{
                graphics::draw(ctx, &self.blood_wall_image, (pnt2,))?;
            }else{
                graphics::draw(ctx, &self.blood_image, (pnt2,))?;
            }
        }

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
                    offset = Point2 {x:0.98, y:0.0};
                    rotation = -std::f32::consts::PI/2.0;
                }
            },
            Direction::Right => {
                if mydir == Direction::Down{
                    offset = Point2 {x:1.0, y:0.0};
                    rotation = -std::f32::consts::PI/2.0;
                }else if mydir == Direction::Up{
                    offset = Point2 { x:0.0, y:-0.02 };
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