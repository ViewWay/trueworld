// Sprite generation and spawning system
//
// This module handles:
// - Procedural sprite generation for different entity types
// - Sprite bundle creation and spawning
// - Color configuration for entities

#![allow(dead_code)]

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use trueworld_core::{EntityType, EntityId};

/// Color configuration for different entity types
#[derive(Resource, Clone)]
pub struct EntityColors {
    pub player: Color,
    pub monster: Color,
    pub prop: Color,
    pub item: Color,
    pub effect: Color,
    pub npc: Color,
}

impl Default for EntityColors {
    fn default() -> Self {
        Self {
            player: Color::srgb(0.2, 0.6, 1.0),    // Blue
            monster: Color::srgb(1.0, 0.2, 0.2),    // Red
            prop: Color::srgb(0.4, 0.8, 0.4),      // Green
            item: Color::srgb(1.0, 0.8, 0.2),      // Yellow/Gold
            effect: Color::srgb(0.8, 0.4, 1.0),    // Purple
            npc: Color::srgb(0.5, 0.5, 0.5),       // Gray
        }
    }
}

/// Procedurally generated sprite textures
#[derive(Resource, Clone)]
pub struct ProceduralSprites {
    pub player: Handle<Image>,
    pub monster: Handle<Image>,
    pub prop: Handle<Image>,
    pub item: Handle<Image>,
    pub effect: Handle<Image>,
    pub npc: Handle<Image>,
}

/// Generate a procedural sprite for a given entity type
pub fn generate_entity_sprite(
    entity_type: EntityType,
    size: UVec2,
    colors: &EntityColors,
) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0], // Transparent background
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let color = match entity_type {
        EntityType::Player => colors.player,
        EntityType::Monster => colors.monster,
        EntityType::Prop => colors.prop,
        EntityType::Item => colors.item,
        EntityType::Projectile => colors.effect,
        EntityType::Effect => colors.effect,
    };

    // Convert Bevy Color to RGBA u8
    let srgba = color.to_srgba();
    let r = (srgba.red * 255.0) as u8;
    let g = (srgba.green * 255.0) as u8;
    let b = (srgba.blue * 255.0) as u8;
    let a = (srgba.alpha * 255.0) as u8;

    // Draw different shapes based on entity type
    match entity_type {
        EntityType::Player => draw_circle(&mut image, r, g, b, a, 0.8),
        EntityType::Monster => draw_square(&mut image, r, g, b, a, 0.7),
        EntityType::Prop => draw_triangle(&mut image, r, g, b, a, 0.9),
        EntityType::Item => draw_diamond(&mut image, r, g, b, a, 0.6),
        EntityType::Projectile => draw_star(&mut image, r, g, b, a, 0.5),
        EntityType::Effect => draw_cross(&mut image, r, g, b, a, 0.7),
    }

    image
}

/// Draw a filled circle
fn draw_circle(image: &mut Image, r: u8, g: u8, b: u8, a: u8, scale: f32) {
    let width = image.width() as f32;
    let height = image.height() as f32;
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let radius = (width.min(height) / 2.0) * scale;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                // Anti-aliased edge
                let alpha = if dist > radius - 2.0 {
                    let edge_factor = (radius - dist) / 2.0;
                    (a as f32 * edge_factor) as u8
                } else {
                    a
                };
                set_pixel(image, x, y, r, g, b, alpha);
            }
        }
    }
}

/// Draw a filled square
fn draw_square(image: &mut Image, r: u8, g: u8, b: u8, a: u8, scale: f32) {
    let width = image.width();
    let height = image.height();
    let size = (width.min(height) as f32 * scale) as u32;
    let offset_x = (width - size) / 2;
    let offset_y = (height - size) / 2;

    for y in offset_y..(offset_y + size) {
        for x in offset_x..(offset_x + size) {
            // Add a border
            let is_border = x == offset_x || x == offset_x + size - 1
                || y == offset_y || y == offset_y + size - 1;
            let border_a = if is_border { a / 2 } else { a };
            set_pixel(image, x, y, r, g, b, border_a);
        }
    }
}

/// Draw a filled triangle (pointing up)
fn draw_triangle(image: &mut Image, r: u8, g: u8, b: u8, a: u8, scale: f32) {
    let width = image.width() as f32;
    let height = image.height() as f32;
    let size = (width.min(height) / 2.0) * scale;

    let center_x = width / 2.0;
    let bottom_y = height - (height - size) / 2.0;
    let top_y = (height - size) / 2.0;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let px = x as f32;
            let py = y as f32;

            // Triangle point-in-polygon test
            let p1 = (center_x, top_y);
            let p2 = (center_x - size, bottom_y);
            let p3 = (center_x + size, bottom_y);

            if point_in_triangle(px, py, p1, p2, p3) {
                set_pixel(image, x, y, r, g, b, a);
            }
        }
    }
}

/// Draw a diamond (rotated square)
fn draw_diamond(image: &mut Image, r: u8, g: u8, b: u8, a: u8, scale: f32) {
    let width = image.width() as f32;
    let height = image.height() as f32;
    let size = (width.min(height) / 2.0) * scale;

    let center_x = width / 2.0;
    let center_y = height / 2.0;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let px = x as f32;
            let py = y as f32;

            // Diamond: |x - center| + |y - center| <= size
            let dx = (px - center_x).abs();
            let dy = (py - center_y).abs();

            if dx + dy <= size / 2.0 {
                set_pixel(image, x, y, r, g, b, a);
            }
        }
    }
}

/// Draw a 5-pointed star
fn draw_star(image: &mut Image, r: u8, g: u8, b: u8, a: u8, scale: f32) {
    let width = image.width() as f32;
    let height = image.height() as f32;
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let outer_radius = (width.min(height) / 2.0) * scale;
    let inner_radius = outer_radius * 0.4;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let px = x as f32;
            let py = y as f32;

            let dx = px - center_x;
            let dy = py - center_y;
            let angle = dy.atan2(dx);
            let dist = (dx * dx + dy * dy).sqrt();

            // Star shape check
            let star_angle = (angle + std::f32::consts::PI / 2.0).rem_euclid(2.0 * std::f32::consts::PI);
            let points = 5.0;
            let segment = star_angle / (2.0 * std::f32::consts::PI / points);
            let fractional = segment - segment.floor();
            let radius = if fractional < 0.5 {
                outer_radius
            } else {
                inner_radius
            };

            // Check if point is near the star outline
            if dist <= radius && dist >= radius - 3.0 {
                set_pixel(image, x, y, r, g, b, a);
            }
        }
    }
}

/// Draw a cross shape
fn draw_cross(image: &mut Image, r: u8, g: u8, b: u8, a: u8, scale: f32) {
    let width = image.width();
    let height = image.height();
    let thickness = ((width.min(height) as f32) * scale * 0.2) as u32;
    let length = ((width.min(height) as f32) * scale * 0.8) as u32;

    let center_x = width / 2;
    let center_y = height / 2;
    let half_thickness = thickness / 2;

    // Vertical bar
    for y in (center_y - length / 2)..(center_y + length / 2) {
        for x in (center_x - half_thickness)..(center_x + half_thickness) {
            if y < height && x < width {
                set_pixel(image, x, y, r, g, b, a);
            }
        }
    }

    // Horizontal bar
    for y in (center_y - half_thickness)..(center_y + half_thickness) {
        for x in (center_x - length / 2)..(center_x + length / 2) {
            if y < height && x < width {
                set_pixel(image, x, y, r, g, b, a);
            }
        }
    }
}

/// Set a pixel in the image
fn set_pixel(image: &mut Image, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let index = ((y * image.width() + x) * 4) as usize;
    if index + 3 < image.data.len() {
        image.data[index] = r;
        image.data[index + 1] = g;
        image.data[index + 2] = b;
        image.data[index + 3] = a;
    }
}

/// Barycentric coordinate test for triangle
fn point_in_triangle(
    px: f32,
    py: f32,
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
) -> bool {
    let denominator = (p2.1 - p3.1) * (p1.0 - p3.0) + (p3.0 - p2.0) * (p1.1 - p3.1);
    if denominator.abs() < 0.001 {
        return false;
    }

    let a = ((p2.1 - p3.1) * (px - p3.0) + (p3.0 - p2.0) * (py - p3.1)) / denominator;
    let b = ((p3.1 - p1.1) * (px - p3.0) + (p1.0 - p3.0) * (py - p3.1)) / denominator;
    let c = 1.0 - a - b;

    a >= 0.0 && b >= 0.0 && c >= 0.0
}

use crate::render::animation::AnimationController;
use crate::render::NetworkEntity;

/// Spawn an entity sprite with all required components
pub fn spawn_entity_sprite(
    commands: &mut Commands,
    entity_id: EntityId,
    entity_type: EntityType,
    position: Vec2,
    sprites: &ProceduralSprites,
    _player_name: Option<String>,
) -> Entity {
    let texture = match entity_type {
        EntityType::Player => sprites.player.clone(),
        EntityType::Monster => sprites.monster.clone(),
        EntityType::Prop => sprites.prop.clone(),
        EntityType::Item => sprites.item.clone(),
        EntityType::Projectile => sprites.effect.clone(),
        EntityType::Effect => sprites.effect.clone(),
    };

    let sprite_size = match entity_type {
        EntityType::Player => Vec2::new(32.0, 32.0),
        EntityType::Monster => Vec2::new(40.0, 40.0),
        EntityType::Prop => Vec2::new(48.0, 48.0),
        EntityType::Item => Vec2::new(16.0, 16.0),
        EntityType::Projectile => Vec2::new(12.0, 12.0),
        EntityType::Effect => Vec2::new(24.0, 24.0),
    };

    let z_index = match entity_type {
        EntityType::Effect => 100,  // Effects on top
        EntityType::Item => 90,
        EntityType::Projectile => 80,
        EntityType::Player => 50,
        EntityType::Monster => 40,
        EntityType::Prop => 10,  // Props at bottom
    };

    commands.spawn((
        Sprite {
            image: texture,
            custom_size: Some(sprite_size),
            ..default()
        },
        Transform::from_translation(position.extend(z_index as f32)),
        Visibility::default(),
        NetworkEntity(entity_id),
        super::sync::NetworkEntityMarker,
        AnimationController::default(),
    )).id()
}

/// Plugin for sprite generation and management
pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityColors>();

        app.add_systems(Startup, setup_sprites);
    }
}

/// Setup procedural sprites on startup
fn setup_sprites(
    mut commands: Commands,
    colors: Res<EntityColors>,
    mut images: ResMut<Assets<Image>>,
) {
    let sprite_size = UVec2::new(64, 64);

    let player = images.add(generate_entity_sprite(EntityType::Player, sprite_size, &colors));
    let monster = images.add(generate_entity_sprite(EntityType::Monster, sprite_size, &colors));
    let prop = images.add(generate_entity_sprite(EntityType::Prop, sprite_size, &colors));
    let item = images.add(generate_entity_sprite(EntityType::Item, sprite_size, &colors));
    let effect = images.add(generate_entity_sprite(EntityType::Effect, sprite_size, &colors));
    let npc = images.add(generate_entity_sprite(EntityType::Monster, sprite_size, &colors));

    commands.insert_resource(ProceduralSprites {
        player,
        monster,
        prop,
        item,
        effect,
        npc,
    });

    info!("Procedural sprites generated successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_colors_default() {
        let colors = EntityColors::default();
        assert_ne!(colors.player, colors.monster);
        assert_ne!(colors.monster, colors.prop);
    }

    #[test]
    fn test_generate_sprite_size() {
        let colors = EntityColors::default();
        let sprite = generate_entity_sprite(EntityType::Player, UVec2::new(32, 32), &colors);
        assert_eq!(sprite.width(), 32);
        assert_eq!(sprite.height(), 32);
    }
}
