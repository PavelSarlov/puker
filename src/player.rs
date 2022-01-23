use ggez::{
    graphics::{self, DrawParam, Color},
    GameResult,
    Context,
    audio::SoundSource,
};
use crate::{
    utils::*,
    assets::*,
    consts::*,
    traits::*,
    shots::*,
};
use glam::f32::{Vec2};
use std::{
    any::Any,
};

#[derive(Clone, Debug, Copy)]
pub struct Player {
    pub props: ActorProps,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub damaged_cooldown: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
}

impl Actor for Player {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);

        if self.afterlock_cooldown == 0. {
            self.velocity_lerp(_delta_time, self.speed, 10., 40.);
        }

        self.props.pos.0 += self.props.velocity;

        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.damaged_cooldown = f32::max(0., self.damaged_cooldown - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            assets.audio.get_mut("player_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("player_base").unwrap().dimensions()))
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Shoot => {
                if self.props.forward == Vec2::X { graphics::draw(ctx, assets.sprites.get("player_shoot_east").unwrap(), draw_params)?; }
                else if self.props.forward == -Vec2::X { graphics::draw(ctx, assets.sprites.get("player_shoot_west").unwrap(), draw_params)?; }
                else if self.props.forward == -Vec2::Y { graphics::draw(ctx, assets.sprites.get("player_shoot_north").unwrap(), draw_params)?; }
                else if self.props.forward == Vec2::Y { graphics::draw(ctx, assets.sprites.get("player_shoot_south").unwrap(), draw_params)?; }
                else { graphics::draw(ctx, assets.sprites.get("player_base").unwrap(), draw_params)?; }
            },
            ActorState::Damaged => {
                assets.audio.get_mut("player_damaged_sound").unwrap().play(ctx)?;
                graphics::draw(ctx, assets.sprites.get("player_damaged").unwrap(), draw_params.color(Color::RED))?;
            },
            ActorState::Dead => graphics::draw(ctx, assets.sprites.get("player_dead").unwrap(), draw_params)?,
            _ => graphics::draw(ctx, assets.sprites.get("player_base").unwrap(), draw_params)?,
        }

        if conf.draw_bcircle_model { self.draw_bcircle(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn set_pos(&mut self, new_pos: Vec2) { self.props.pos = new_pos.into(); }

    fn set_scale(&mut self, new_scale: Vec2) { self.props.scale = new_scale; }

    fn set_velocity(&mut self, new_velocity: Vec2) { self.props.velocity = new_velocity; } 

    fn set_translation(&mut self, new_translation: Vec2) { self.props.translation = new_translation; }

    fn set_forward(&mut self, new_forward: Vec2) { self.props.forward = new_forward; } 

    fn get_health(&self) -> f32 { self.health }

    fn get_state(&self) -> ActorState { self.state }

    fn damage(&mut self, dmg: f32) { 
        if self.damaged_cooldown <= 0. {
            self.health -= dmg; 
            self.state = ActorState::Damaged;
            self.damaged_cooldown = PLAYER_DAMAGED_COOLDOWN;
            self.animation_cooldown = ANIMATION_COOLDOWN / self.damaged_cooldown;
        }
    }

    fn get_tag(&self) -> ActorTag { ActorTag::Player }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Player {
    pub fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. {
            return Ok(());
        }

        if self.state != ActorState::Shoot {
            self.state = ActorState::Shoot;
            self.animation_cooldown = ANIMATION_COOLDOWN / self.shoot_rate;
        }

        self.shoot_timeout = 1. / self.shoot_rate;
        let shot_dir = (self.props.forward + 0.5 * (self.props.velocity.clamp_length_max(0.5) * Vec2::new(self.props.forward.y, self.props.forward.x).abs())).normalize();

        let shot = Shot {
            props: ActorProps {
                pos: self.props.pos,
                scale: Vec2::splat(SHOT_SCALE),
                translation: shot_dir,
                forward: shot_dir,
                velocity: Vec2::ZERO,
            },
            spawn_pos: self.props.pos,
            speed: SHOT_SPEED,
            range: self.shoot_range,
            damage: self.damage,
            tag: ShotTag::Player,
        };

        shots.push(shot);

        Ok(())
    }
}