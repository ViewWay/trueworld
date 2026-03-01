// crates/client/src/app.rs

use bevy::{
    app::{App, Startup},
    asset::AssetPlugin,
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    scene::ScenePlugin,
    time::TimePlugin,
};

use crate::{
    input::InputPlugin as ClientInputPlugin,
    network::NetworkPlugin,
    render::EntityRenderPlugin,
    state::GameState,
};

/// TrueWorld 客户端应用
pub struct TrueWorldClient {
    app: App,
}

impl TrueWorldClient {
    pub fn new() -> anyhow::Result<Self> {
        let mut app = App::new();

        // 渲染配置 (Bevy 0.15)
        let render_creation = RenderCreation::Automatic(WgpuSettings {
            backends: Some(Backends::all()),
            ..Default::default()
        });

        // 基础插件
        app.add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation,
            ..Default::default()
        }));

        // 额外插件
        app.add_plugins((
            TimePlugin,
            AssetPlugin::default(),
            ScenePlugin,
            DiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ));

        // 自定义插件
        app.add_plugins((
            ClientInputPlugin,
            NetworkPlugin,
            EntityRenderPlugin,
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
fn setup(mut commands: Commands) {
    info!("TrueWorld Client initialized");

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

    // 2D 相机
    commands.spawn(Camera2d);

    // 3D 相机
    commands.spawn(Camera3d::default());
}
