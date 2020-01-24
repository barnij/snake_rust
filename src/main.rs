use ggez;

use ggez::event::{KeyCode, KeyMods};
use ggez::{ event,
            graphics,
            Context,
            GameResult};

use std::time::{Duration, Instant};

use ggez::mint::Point2;

mod consts;
use consts::*;

mod window;
use window::build_window;

mod elements;
use elements::*;


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
                    let pnt2: Point2<f32> = gp.into();
                    graphics::draw(ctx, &self.floor_image, (pnt2,))?;
                }
            }
        }
        Ok(())
    }

    fn draw_score(&mut self, ctx: &mut Context) -> GameResult {
        let copy_txt = self.points_text.clone().add(self.points.to_string()).to_owned();
        let gp: GridPosition = (5 as i16, 0 as i16).into();
        let pnt2: Point2<f32> = gp.into();
        graphics::draw(ctx, &copy_txt, (pnt2,))?;
        Ok(())
    }

    fn draw_game_over(&mut self, ctx: &mut Context) -> GameResult{
        let text = graphics::Text::new("GAME OVER")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:100.0, y:100.0} ).to_owned();
        let little_text = graphics::Text::new("PRESS R TO RESTART OR ESCAPE TO EXIT")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:20.0, y:20.0} ).to_owned();
        let gp: GridPosition = (10 as i16, 8 as i16).into();
        let mut pnt2: Point2<f32> = gp.into();
        pnt2.x -= 20.0;
        let gp1: GridPosition = (10 as i16, 10 as i16).into();
        let mut pnt2_1: Point2<f32> = gp1.into();
        pnt2_1.y += 15.0;
        pnt2_1.x += 20.0;
        graphics::draw(ctx, &text, (pnt2,))?;
        graphics::draw(ctx, &little_text, (pnt2_1,))?;
        Ok(())
    }

    fn draw_start(&mut self, ctx: &mut Context) -> GameResult{
        let text = graphics::Text::new("PRESS SPACE TO START THE GAME")
                                        .set_font( graphics::Font::new(ctx,"/Terminus.ttf").unwrap(),
                                                   graphics::Scale{x:30.0, y:30.0} ).to_owned();
        let gp: GridPosition = (14 as i16, 0 as i16).into();
        let mut pnt2: Point2<f32> = gp.into();
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

    let (ctx, events_loop) = &mut build_window().build()?;

    let state = &mut GameState::new(ctx).unwrap();

    event::run(ctx, events_loop, state)
}