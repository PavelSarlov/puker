use crate::{
    player::*,
    traits::*,
    utils::*,
    consts::*,
};
use ggez::{
    graphics::{self, DrawParam},
    Context,
    GameResult,
};
use glam::f32::Vec2;
use std::any::Any;

#[derive(Debug, PartialEq)]
pub enum CollectableTag {
    RedHeart(f32),
    SpeedBoost(f32),
    ShootRateBoost(f32),
    DamageBoost(f32),
}

#[derive(Debug, PartialEq)]
pub enum CollectableState {
    Base,
    Consumed,
}

#[derive(Debug)]
pub struct Collectable {
    pub props: ActorProps,
    pub tag: CollectableTag,
    pub state: CollectableState,
}

impl Actor for Collectable {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config, _delta_time: f32) -> GameResult {
        self.velocity_lerp(_delta_time, 4., 0.01, 0.);

        self.props.pos.0 += self.props.velocity;

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let sprite = match self.tag {
            CollectableTag::RedHeart(a) => {
                if a == 1. {
                    conf.assets.sprites.get("heart_full_collectable").unwrap()
                } else {
                    conf.assets.sprites.get("heart_half_collectable").unwrap()
                }
            },
            CollectableTag::SpeedBoost(_) => conf.assets.sprites.get("speed_boost").unwrap(),
            CollectableTag::ShootRateBoost(_) => conf.assets.sprites.get("shoot_rate_boost").unwrap(),
            CollectableTag::DamageBoost(_) => conf.assets.sprites.get("damage_boost").unwrap(),
        };

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, sprite.dimensions()))
            .offset([0.5, 0.5]);

        graphics::draw(ctx, sprite, draw_params)?;

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

    fn get_health(&self) -> f32 { 0. }

    fn get_state(&self) -> ActorState { ActorState::Base }

    fn get_tag(&self) -> ActorTag { ActorTag::Player }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Collectable {
    pub fn affect_player(&mut self, player: &mut Player) -> bool {
        if self.state == CollectableState::Consumed {
            return false;
        }

        match self.tag {
            CollectableTag::RedHeart(h) => {
                if player.health < player.max_health {
                    player.health = f32::min(player.health + h, player.max_health);
                }
                else {
                    return false;
                }
            }
            CollectableTag::ShootRateBoost(b) => {
                player.shoot_rate = f32::min(player.shoot_rate * b, PLAYER_MAX_SHOOT_RATE);
            }
            CollectableTag::SpeedBoost(b) => {
                player.speed = f32::min(player.speed * b, PLAYER_MAX_SPEED);
            }
            CollectableTag::DamageBoost(b) => {
                player.damage = f32::min(player.damage * b, PLAYER_MAX_DAMAGE);
            }
        };

        self.state = CollectableState::Consumed; 

        true
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ItemPassive {
    IncreaseMaxHealth(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum ItemActive {
    Heal(f32),
}

#[derive(Debug, Copy, Clone)]
pub enum ItemTag {
    Passive(ItemPassive),
    Active(ItemActive),
}

#[derive(Debug, Copy, Clone)]
pub struct Item {
    pub tag: ItemTag,
    pub cooldown: f32,
}

impl Item {
    pub fn affect_player(&mut self, player: &mut Player) {
        match self.tag {
            ItemTag::Passive(p) => match p {
                ItemPassive::IncreaseMaxHealth(x) => player.max_health += x,
            },
            _ => (),
        }
    }
    
    pub fn activate(&mut self, player: &mut Player) -> bool {
        if self.cooldown > 0. { return false; } 
        self.cooldown = ITEM_COOLDOWN;

        match self.tag {
            ItemTag::Active(a) => match a {
                ItemActive::Heal(x) => player.health = (player.health + x).clamp(0., player.max_health),
            },
            _ => (),
        }

        true
    }
}
