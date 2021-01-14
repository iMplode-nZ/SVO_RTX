#![feature(array_map)]
#![feature(const_in_array_repeat_expressions)]
use crate::raytracer::{OctreeRayTracerPlugin};
use bevy::render::draw::DrawContext;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::{
    BlendDescriptor, ColorStateDescriptor, ColorWrite, IndexFormat, PrimitiveTopology,
};
use bevy::render::texture::TextureFormat;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use svo::octree::Octree;
use crate::raytracer::chunk::{Chunk, Voxel, ChunkBundle};
use bevy::render::camera::PerspectiveProjection;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsPlugin};
use crate::lights::SunLight;
use bevy::asset::HandleId;
use crate::material::texture_repo::TextureRepo;
use crate::material::{MaterialPalette, DEFAULT_MATERIAL_PALETTE_HANDLE, Material};

mod raytracer;
mod lights;
mod material;

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        // Bevy plugins
        .add_plugin(bevy::reflect::ReflectPlugin::default())
        .add_plugin(bevy::core::CorePlugin::default())
        .add_plugin(bevy::transform::TransformPlugin::default())
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin::default())
        .add_plugin(bevy::input::InputPlugin::default())
        .add_plugin(bevy::window::WindowPlugin::default())
        .add_plugin(bevy::asset::AssetPlugin::default())
        .add_plugin(bevy::render::RenderPlugin::default())
        .add_plugin(bevy::winit::WinitPlugin::default())
        .add_plugin(bevy::wgpu::WgpuPlugin::default())
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Custom plugins
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_plugin(OctreeRayTracerPlugin::default())
        .add_system(my_system.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut chunks: ResMut<Assets<Chunk>>,
    mut render_graph: ResMut<RenderGraph>,
    mut texture_repo: ResMut<TextureRepo>,
    mut material_palettes: ResMut<Assets<MaterialPalette>>,
    mut materials: ResMut<Assets<Material>>
) {
    let grass_material = materials.add(Material {
        name: "Grass".into(),
        diffuse: texture_repo.load("assets/textures/grass.jpg"),
    });
    let rock_material = materials.add(Material {
        name: "Rock".into(),
        diffuse: texture_repo.load("assets/textures/rock.jpg"),
    });

    let mut palette = material_palettes.get_mut(DEFAULT_MATERIAL_PALETTE_HANDLE).unwrap();
    let grass_voxel = palette.add_material(grass_material);
    let rock_voxel = palette.add_material(rock_material);


    let lod = 4;

    let mut octree2: Octree<Voxel> = Octree::new();
    let monument = dot_vox::load("assets/monu9.vox").unwrap();
    let model = &monument.models[0];

    for (i, color) in monument.palette.iter().enumerate() {
        let r = color >> 24;
        let g = (color >> 16) & 0xFF;
        let b = (color >> 8 ) & 0xFF;
        let a = color & 0xFF;
        palette.color_palette[i] = Color::rgba_u8(r as u8, g as u8, b as u8, a as u8);
    }
    println!("Added material");



    for voxel in &model.voxels {
        let v = if voxel.i == 58 { grass_voxel } else { rock_voxel };
        octree2.set(voxel.x as u32, voxel.z as u32, voxel.y as u32, 256, v);
    }



    let chunk2 = Chunk::new(octree2, Vec4::new(0.0, 0.0, 0.0, 16.0));

    let chunk_handle2 = chunks.add(chunk2);
    commands
        .spawn(ChunkBundle::new(chunk_handle2))
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            perspective_projection: PerspectiveProjection {
                near: 0.1,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(FlyCamera::default());
}


fn my_system(
    mut sun_light_resource: ResMut<SunLight>,
    time: Res<Time>
) {
    sun_light_resource.direction.x = (time.seconds_since_startup()).cos() as f32;
    sun_light_resource.direction.z = (time.seconds_since_startup()).sin() as f32;
}