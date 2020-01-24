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

    fn draw(&self, ctx: &mut Context, gameover: bool) -> GameResult<()> {

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


struct GameState {
    snake: Snake,
    food: Food,
    walls: Wall,
    gameover: bool,
    start: bool,
    points: u32,
    points_text: graphics::Text,
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
            start: false,
            points: 0,
            points_text: graphics::Text::new("Points: ")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:35.0, y:38.0} ).to_owned(),
            last_update: Instant::now(),
            floor_image: image,
        };

        Ok(s)
    }

    fn draw_floor(&mut self, ctx: &mut Context) -> GameResult {
        for i in 0..GRID_SIZE.0{
            for j in 0..GRID_SIZE.1{
                if j != 0{
                    let gp: GridPosition = (i as i16, j as i16).into();
                    let pnt2: ggez::mint::Point2<f32> = gp.into();
                    graphics::draw(ctx, &self.floor_image, (pnt2,))?;
                }
            }
        }
        Ok(())
    }

    fn draw_score(&mut self, ctx: &mut Context) -> GameResult {
        let copy_txt = self.points_text.clone().add(self.points.to_string()).to_owned();
        let gp: GridPosition = (5 as i16, 0 as i16).into();
        let pnt2: ggez::mint::Point2<f32> = gp.into();
        graphics::draw(ctx, &copy_txt, (pnt2,))?;
        Ok(())
    }

    fn draw_game_over(&mut self, ctx: &mut Context) -> GameResult{
        let text = graphics::Text::new("GAME OVER")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:100.0, y:100.0} ).to_owned();
        let little_text = graphics::Text::new("PRESS R TO RESTART OR ANY KEY TO EXIT")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:20.0, y:20.0} ).to_owned();
        let gp: GridPosition = (10 as i16, 8 as i16).into();
        let mut pnt2: ggez::mint::Point2<f32> = gp.into();
        pnt2.x -= 20.0;
        let gp1: GridPosition = (10 as i16, 10 as i16).into();
        let mut pnt2_1: ggez::mint::Point2<f32> = gp1.into();
        pnt2_1.y += 15.0;
        pnt2_1.x += 15.0;
        graphics::draw(ctx, &text, (pnt2,))?;
        graphics::draw(ctx, &little_text, (pnt2_1,))?;
        Ok(())
    }

    fn draw_start(&mut self, ctx: &mut Context) -> GameResult{
        let text = graphics::Text::new("PRESS SPACE TO START THE GAME")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:30.0, y:30.0} ).to_owned();
        let gp: GridPosition = (14 as i16, 0 as i16).into();
        let mut pnt2: ggez::mint::Point2<f32> = gp.into();
        pnt2.y += 5.0;
        graphics::draw(ctx, &text, (pnt2,))?;
        Ok(())
    }

    fn is_ready_for_tick(&mut self) -> bool {
        Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE)
    }

    fn restart_game(&mut self, ctx: &mut Context) {
        let snake_pos = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();
        let food_pos = GridPosition::random(1, 1, GRID_SIZE.0 - 1, GRID_SIZE.1 - 1);

        self.snake = Snake::new(snake_pos, ctx).unwrap();
        self.food = Food::new(food_pos, ctx).unwrap();
        self.walls = Wall::new(ctx).unwrap();
        self.gameover = false;
        self.start = false;
        self.points = 0;
        self.last_update = Instant::now();
    }

}

impl event::EventHandler for GameState {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        if self.is_ready_for_tick() {
            if !self.gameover && self.start {

                self.snake.update(&self.food, &self.walls);

                if let Some(ate) = self.snake.ate {
                    match ate {
                        Ate::Food => {
                            let new_food_pos = GridPosition::random(2, 3, GRID_SIZE.0 - 1, GRID_SIZE.1 - 1);
                            self.food.pos = new_food_pos;
                            self.points += 1;
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

        self.draw_floor(ctx)?;
        self.walls.draw(ctx)?;
        self.snake.draw(ctx, self.gameover)?;
        self.food.draw(ctx)?;
        self.draw_score(ctx)?;
        if self.gameover{
            self.draw_game_over(ctx)?;
        }
        if !self.start{
            self.draw_start(ctx)?;
        }

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

        if self.start{

            if let Some(dir) = Direction::from_keycode(keycode) {

                if self.snake.dir != self.snake.last_update_dir && dir.inverse() != self.snake.dir {
                    self.snake.next_dir = Some(dir);
                } else if dir.inverse() != self.snake.last_update_dir {
                    self.snake.next_dir = Some(dir);
                }
            }

            if keycode == KeyCode::Escape {
                event::quit(_ctx);
            }else if keycode == KeyCode::R {
                self.restart_game(_ctx);
            }else if self.gameover{
                event::quit(_ctx);
            }

        }else{
             if keycode == KeyCode::Escape {
                event::quit(_ctx);
            }else if keycode == KeyCode::Space {
                self.start = true;
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
        .window_setup(ggez::conf::WindowSetup::default().title("Snake in Rust - project").vsync(true))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1).fullscreen_type(ggez::conf::FullscreenType::Windowed).resizable(true))
        .build()?;


    let state = &mut GameState::new(ctx).unwrap();

    event::run(ctx, events_loop, state)
}