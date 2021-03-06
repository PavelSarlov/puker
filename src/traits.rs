use ggez::{
    graphics::{self, Rect, DrawParam, PxScale, DrawMode, Mesh, Text, Color},
    GameResult,
    Context,
    mint::{Point2},
    event::{KeyCode, MouseButton},
    input,
};
use std::{
    any::Any,
    cell::{Ref, RefMut},
};
use glam::f32::Vec2;
use crate::{
    consts::*,
    utils::*,
    player::*,
    shots::Shot,
    dungeon::BlockTag,
};
use rand::{thread_rng, Rng};

pub trait Actor: std::fmt::Debug {
    fn update(&mut self, _ctx: &mut Context, _config: &mut Config, _delta_time: f32) -> GameResult { Ok(()) }

    fn draw(&self, _ctx: &mut Context, _config: &mut Config) -> GameResult { Ok(()) }

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale::from(24.0))).count();
        graphics::draw(ctx, &text, DrawParam::default().dest([bbox.x, bbox.y - text.height(ctx)]))?;

        Ok(())
    }

    fn draw_bcircle(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh)= screen;
        let bcircle = self.get_bcircle(sw, sh);
        let mut text = Text::new(format!("{:?}", bcircle.0.0));

        let mesh = Mesh::new_circle(ctx, DrawMode::stroke(2.0), bcircle.0, bcircle.1, 0.5, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale::from(24.0))).count();
        graphics::draw(ctx, &text, DrawParam::default().dest([bcircle.0.0.x + bcircle.1, bcircle.0.0.y]))?;

        Ok(())
    }

    fn get_bcircle(&self, sw: f32, sh: f32) -> (Vec2Wrap, f32) {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        (Vec2::new(self.get_pos().x, self.get_pos().y).into(), f32::min(width, height) / 2.)
    }    

    fn get_bbox(&self, sw: f32, sh: f32) -> graphics::Rect {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y - height / 2., width, height)
    }

    fn scale_to_screen(&self, sw: f32, sh: f32, image: Rect) -> Vec2 {
        let bbox = self.get_bbox(sw, sh);
        Vec2::new(bbox.w / image.w, bbox.h / image.h)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn get_velocity(&self) -> Vec2;

    fn get_translation(&self) -> Vec2;

    fn get_forward(&self) -> Vec2;

    fn set_pos(&mut self, _new_pos: Vec2) {}

    fn set_scale(&mut self, _new_scale: Vec2) {}

    fn set_velocity(&mut self, _new_velocity: Vec2) {}

    fn set_translation(&mut self, _new_translation: Vec2) {}

    fn set_forward(&mut self, _new_forward: Vec2) {}

    fn velocity_lerp(&mut self, _delta_time: f32, speed: f32, decay: f32, acceleration: f32) {
        self.set_translation(self.get_translation().clamp_length_max(1.));
        self.set_velocity(self.get_velocity() + self.get_translation() * acceleration * _delta_time);
        if self.get_translation().length() == 0. { self.set_velocity(self.get_velocity() / decay.clamp(_delta_time * (1. + 1e-2), f32::INFINITY) * _delta_time) }
        if self.get_velocity().length() < 0.01 { self.set_velocity(Vec2::ZERO); }
        if self.get_velocity().length() > speed && speed > 0. { self.set_velocity(self.get_velocity().clamp_length_max(speed)); }
    }

    fn act(&mut self, _sw: f32, _sh: f32, _grid: &[[i32; ROOM_WIDTH]], _obstacles: &Vec<Box<dyn Stationary>>, _shots: &mut Vec<Shot>, _player: &Player) -> GameResult { Ok(()) }

    fn get_health(&self) -> f32; 

    fn get_state(&self) -> ActorState;

    fn damage(&mut self, _dmg: f32) {}

    fn get_damage(&self) -> f32 { 0. }

    fn get_tag(&self) -> ActorTag;

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Stationary: std::fmt::Debug {
    fn update(&mut self, _config: &mut Config, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, _config: &mut Config) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale { x: 0.5, y: 0.5 })).count();
        graphics::draw(ctx, &text, DrawParam::default().dest([bbox.x, bbox.y - text.height(ctx)]))?;

        Ok(())
    }

    fn draw_bcircle(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh)= screen;
        let bcircle = self.get_bcircle(sw, sh);
        let mut text = Text::new(format!("{:?}", bcircle.0.0));

        let mesh = Mesh::new_circle(ctx, DrawMode::stroke(2.0), bcircle.0, bcircle.1, 0.5, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale::from(24.0))).count();
        graphics::draw(ctx, &text, DrawParam::default().dest([bcircle.0.0.x + bcircle.1, bcircle.0.0.y]))?;

        Ok(())
    }

    fn get_bbox(&self, sw: f32, sh: f32) -> graphics::Rect {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y - height / 2., width, height)
    }

    fn get_bcircle(&self, sw: f32, sh: f32) -> (Vec2Wrap, f32) {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        (Vec2::new(self.get_pos().x, self.get_pos().y).into(), f32::min(width, height) / 2.)
    }    

    fn scale_to_screen(&self, sw: f32, sh: f32, image: Rect) -> Vec2 {
        let bbox = self.get_bbox(sw, sh);
        Vec2::new(bbox.w / image.w, bbox.h / image.h)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn get_tag(&self) -> BlockTag;

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult;

    fn draw(&mut self, ctx: &mut Context) -> GameResult;

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {}

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {}

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {}

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { None }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { None }  

    fn get_overlapped_idx(&self, ctx: &mut Context, mx: f32, my: f32) -> Option<usize> {
        let conf = &self.get_conf().unwrap();
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (ww, wh) = (conf.window_width, conf.window_height);

        match self.get_ui_elements() {
            Some(ue) => {
                for (i, e) in ue.iter().enumerate() {
                    if e.mouse_overlap(ctx, mx, my, sw, sh, ww, wh) {
                        return Some(i);
                    }
                }
                None
            },
            None => None,
        }
    }

    fn update_ui_vars(&mut self, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn get_conf(&self) -> Option<Ref<Config>> { None }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { None }

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait UIElement {
    fn update(&mut self, ctx: &mut Context, _conf: &mut Config) -> GameResult;

    fn draw(&mut self, ctx: &mut Context, _conf: &mut Config) -> GameResult;

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32>;

    fn width(&self, ctx: &mut Context, sw: f32) -> f32;

    fn height(&self, ctx: &mut Context, sh: f32) -> f32;

    fn top_left(&self, ctx: &mut Context, sw: f32, sh: f32) -> Vec2 {
        let pos = self.pos(sw, sh);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        Vec2::new(pos.x - w / 2., pos.y - h / 2.)
    }
        
    fn mouse_overlap(&self, ctx: &mut Context, mx: f32, my: f32, sw: f32, sh: f32, ww: f32, wh: f32) -> bool {
        let tl = self.top_left(ctx, sw, sh);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        Rect::new(tl.x, tl.y, w, h).contains(get_mouse_screen_coords(mx, my, sw, sh, ww, wh))
    }

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Chaser: Actor {
    fn chase(&mut self, sw: f32, sh: f32, obstacles: &Vec<Box<dyn Stationary>>, grid: &[[i32; ROOM_WIDTH]], player: &Player);

    fn find_path(&mut self, grid: &[[i32; ROOM_WIDTH]], sw: f32, sh: f32) -> Vec2 {
        let (mut i, mut j) = pos_to_room_coords(self.get_pos(), sw, sh);

        if      i > 0               && grid[i - 1][j] > grid[i][j] { i -= 1; }
        else if j > 0               && grid[i][j - 1] > grid[i][j] { j -= 1; }
        else if j < ROOM_WIDTH - 1  && grid[i][j + 1] > grid[i][j] { j += 1; }
        else if i < ROOM_HEIGHT - 1 && grid[i + 1][j] > grid[i][j] { i += 1; }
        else { return self.get_pos(); }

        room_coords_to_pos(i, j, sw, sh)
    }
}

pub trait Shooter: Actor {
    fn shoot(&mut self, sw: f32, sh: f32, obstacles: &Vec<Box<dyn Stationary>>, shots: &mut Vec<Shot>, player: &Player);

    fn get_range(&self) -> f32;  

    fn get_rate(&self) -> f32;  
}

pub trait Wanderer: Actor {
    fn wander(&mut self, sw: f32, sh: f32, grid: &[[i32; ROOM_WIDTH]]) {
        let (i, j) = pos_to_room_coords(self.get_pos(), sw, sh);
        let mut dirs = Vec::new();

        if i > 0               && grid[i - 1][j] >= 0 { dirs.push(-Vec2::Y); }
        if j > 0               && grid[i][j - 1] >= 0 { dirs.push(-Vec2::X); }
        if j < ROOM_WIDTH - 1  && grid[i][j + 1] >= 0 { dirs.push(Vec2::X); }
        if i < ROOM_HEIGHT - 1 && grid[i + 1][j] >= 0 { dirs.push(Vec2::Y); }

        if (self.get_change_direction_cooldown() == 0. && thread_rng().gen_bool(0.8)) || !dirs.contains(&self.get_translation()) {
            let dir = thread_rng().gen_range(0..dirs.len());
            self.set_translation(dirs[dir]);
            self.set_forward(dirs[dir]);
            self.set_change_direction_cooldown(ENEMY_WANDERER_CHANGE_DIRECTION_COOLDOWN);
        }
    }

    fn get_change_direction_cooldown(&self) -> f32;

    fn set_change_direction_cooldown(&mut self, cd: f32);
}
