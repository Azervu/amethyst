//! Demonstrates loading custom assets using the Amethyst engine.
// TODO: Add asset loader directory store for the meshes.

use amethyst::{
    assets::{DefaultLoader, Format as AssetFormat, Handle, Loader},
    core::{
        math::Vector3,
        transform::{Transform, TransformBundle},
    },
    ecs::World,
    error::Error,
    input::InputBundle,
    prelude::*,
    renderer::{
        camera::Camera,
        light::{Light, PointLight},
        mtl::{Material, MaterialDefaults},
        palette::{Srgb, Srgba},
        plugins::{RenderShaded3D, RenderSkybox, RenderToWindow},
        rendy::{
            mesh::{MeshBuilder, Normal, Position, TexCoord},
            texture::palette::load_from_srgba,
        },
        types::{DefaultBackend, Mesh, MeshData},
        RenderingBundle,
    },
    utils::application_root_dir,
};
use amethyst_assets::{AssetHandle, LoaderBundle, ProcessingQueue};
use amethyst_rendy::{types::TextureData, Texture};
use log::info;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
#[derive(Default, Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "f245dc2b-88a9-413e-bd51-f6c341c32017"]
struct Custom;

amethyst_assets::register_importer!(".custom", Custom);
impl AssetFormat<MeshData> for Custom {
    fn name(&self) -> &'static str {
        "CUSTOM"
    }

    /// Reads the given bytes and produces asset data.
    fn import_simple(&self, bytes: Vec<u8>) -> Result<MeshData, Error> {
        let data: String = String::from_utf8(bytes)?;
        let trimmed: Vec<&str> = data.lines().filter(|line| !line.is_empty()).collect();

        let mut pos = Vec::with_capacity(trimmed.len() * 3);
        let mut norm = Vec::with_capacity(trimmed.len() * 3);
        let mut tex = Vec::with_capacity(trimmed.len() * 3);

        for line in trimmed {
            let nums: Vec<&str> = line.split_whitespace().collect();
            pos.push(Position([
                nums[0].parse::<f32>().unwrap(),
                nums[1].parse::<f32>().unwrap(),
                nums[2].parse::<f32>().unwrap(),
            ]));
            norm.push(Normal([
                nums[3].parse::<f32>().unwrap(),
                nums[4].parse::<f32>().unwrap(),
                nums[5].parse::<f32>().unwrap(),
            ]));
            tex.push(TexCoord([0.0, 0.0]))
        }
        info!("Creating mesh");
        Ok(MeshBuilder::new()
            .with_vertices(pos)
            .with_vertices(norm)
            .with_vertices(tex)
            .into())
    }
}

struct AssetsExample;

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        world.push((0usize,));

        initialise_camera(world);
        initialise_lights(world);

        // Add custom cube object to scene
        let (mesh, mtl) = {
            let mat_defaults = resources.get::<MaterialDefaults>().unwrap();
            let loader = resources.get::<DefaultLoader>().unwrap();

            let textures = &resources.get::<ProcessingQueue<TextureData>>().unwrap();
            let materials = &resources.get().unwrap();

            let mesh: Handle<Mesh> = loader.load("mesh/cuboid.custom");
            let albedo: Handle<Texture> = loader.load_from_data(
                load_from_srgba(Srgba::new(0.1, 0.5, 0.3, 1.0)).into(),
                (),
                textures,
            );
            let mat: Handle<Material> = loader.load_from_data(
                Material {
                    albedo,
                    ..mat_defaults.0.clone()
                },
                (),
                materials,
            );

            (mesh, mat)
        };
        log::debug!(
            "Handle<Mesh>: {:#?}, LoadHandle: {:?}",
            mesh,
            mesh.load_handle()
        );
        let mut trans = Transform::default();
        trans.set_translation_xyz(-5.0, 0.0, 0.0);
        trans.set_scale(Vector3::new(2.0, 2.0, 2.0));
        world.push((mesh, mtl, trans));
    }
}

fn main() -> Result<(), Error> {
    {
        let config = amethyst::LoggerConfig {
            log_file: Some(std::path::PathBuf::from("asset_loading.log")),
            level_filter: amethyst::LogLevelFilter::Info,
            module_levels: vec![
                (
                    "amethyst_assets".to_string(),
                    amethyst::LogLevelFilter::Debug,
                ),
                (
                    "atelier_daemon".to_string(),
                    amethyst::LogLevelFilter::Debug,
                ),
                (
                    "atelier_loader".to_string(),
                    amethyst::LogLevelFilter::Trace,
                ),
            ],
            ..Default::default()
        };
        amethyst::start_logger(config);
    }

    let app_root = application_root_dir()?;
    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("examples/asset_loading/assets/");

    let display_config_path = app_root.join("examples/asset_loading/config/display.ron");

    let mut dispatcher_builder = DispatcherBuilder::default();
    dispatcher_builder
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(InputBundle::new())
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderSkybox::with_colors(
                    Srgb::new(0.82, 0.51, 0.50),
                    Srgb::new(0.18, 0.11, 0.85),
                )),
        );
    let game = Application::new(assets_dir, AssetsExample, dispatcher_builder)?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, -20.0, 10.0);
    transform.prepend_rotation_x_axis(1.325_752_1);

    world.push((
        Camera::perspective(1.0, std::f32::consts::FRAC_PI_3, 0.1),
        transform,
    ));
}

/// Adds lights to the scene.
fn initialise_lights(world: &mut World) {
    let light: Light = PointLight {
        intensity: 100.0,
        radius: 1.0,
        color: Srgb::new(1.0, 1.0, 1.0),
        ..Default::default()
    }
    .into();

    let mut transform = Transform::default();
    transform.set_translation_xyz(5.0, -20.0, 15.0);

    // Add point light.
    world.push((light, transform));
}
