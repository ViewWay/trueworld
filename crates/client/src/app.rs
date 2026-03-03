// crates/client/src/app.rs

use bevy::{app::App, prelude::*};
use trueworld_core::EntityType;

use crate::{
    input::InputPlugin as ClientInputPlugin,
    network::NetworkPlugin,
    render::{EntityRenderPlugin, sprite::EntityColors, sprite::generate_entity_sprite},
    connection::ConnectionPlugin,
    net_sync::NetSyncPlugin,
    movement::ClientMovementPlugin,
    state::GameState,
};

/// TrueWorld 客户端应用
pub struct TrueWorldClient {
    app: App,
}

impl TrueWorldClient {
    pub fn new() -> anyhow::Result<Self> {
        let mut app = App::new();

        // 使用默认插件 (包含窗口支持)
        app.add_plugins(DefaultPlugins);

        // 自定义插件
        app.add_plugins((
            ClientInputPlugin,
            NetworkPlugin,
            EntityRenderPlugin,
            ConnectionPlugin,
            NetSyncPlugin,
            ClientMovementPlugin,
        ));

        // 添加游戏状态 (Bevy 0.15)
        app.init_state::<GameState>();

        Ok(Self { app })
    }

    pub fn run(mut self) {
        // 启动时系统
        self.app.add_systems(Startup, setup);

        self.app.run();
    }
}

/// 初始化系统
fn setup(
    mut commands: Commands,
    colors: Res<EntityColors>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("TrueWorld Client initialized");

    // 设置背景颜色为深色
    commands.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)));

    // 添加环境光
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // 添加默认光照
    commands.spawn(DirectionalLight {
        color: Color::WHITE,
        illuminance: 5000.0,
        shadows_enabled: true,
        ..default()
    });

    // 生成测试精灵
    let sprite_size = bevy::math::UVec2::new(64, 64);
    let player_sprite = images.add(generate_entity_sprite(EntityType::Player, sprite_size, &colors));
    let prop_sprite = images.add(generate_entity_sprite(EntityType::Prop, sprite_size, &colors));

    // 创建一个测试玩家实体 (方块)
    commands.spawn((
        Sprite {
            image: player_sprite,
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        // 标记为本地玩家 (NetworkEntity 是元组结构体)
        crate::render::sync::NetworkEntity(trueworld_core::EntityId::new(1)),
    ));

    // 创建一些测试地面方块
    for x in -5..=5 {
        for y in -3..=3 {
            if x == 0 && y == 0 {
                continue; // 跳过玩家位置
            }
            commands.spawn((
                Sprite {
                    image: prop_sprite.clone(),
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x as f32 * 32.0, y as f32 * 32.0, -0.1)),
            ));
        }
    }

    // 注意：相机由 CameraPlugin 在 Startup 中创建，不需要在这里重复创建
}
