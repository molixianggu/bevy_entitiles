use bevy::{
    asset::AssetId,
    ecs::{component::Component, entity::EntityHashMap, system::Resource, world::FromWorld},
    render::{
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries,
            SamplerBindingType, ShaderStages, TextureSampleType,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
    utils::HashMap,
};

use super::{
    buffer::{
        PerTilemapBuffersStorage, StandardMaterialUniform, StandardMaterialUniformBuffer,
        TilemapStorageBuffers, TilemapUniform, TilemapUniformBuffer, UniformBuffer,
    },
    material::StandardTilemapMaterial,
    pipeline::EntiTilesPipeline,
    texture::TilemapTexturesStorage,
};

use bevy::render::render_resource::binding_types as binding;

#[derive(Component)]
pub struct TilemapViewBindGroup {
    pub value: BindGroup,
}

#[derive(Resource)]
pub struct TilemapBindGroups {
    pub tilemap_uniform_buffer: Option<BindGroup>,
    pub storage_buffers: EntityHashMap<BindGroup>,
    pub materials: HashMap<AssetId<StandardTilemapMaterial>, BindGroup>,
}

impl Default for TilemapBindGroups {
    fn default() -> Self {
        Self {
            tilemap_uniform_buffer: Default::default(),
            storage_buffers: Default::default(),
            materials: Default::default(),
        }
    }
}

impl TilemapBindGroups {
    pub fn bind_uniform_buffers(
        &mut self,
        render_device: &RenderDevice,
        uniform_buffers: &mut TilemapUniformBuffer,
        entitiles_pipeline: &EntiTilesPipeline,
        std_material_uniform_buffer: &StandardMaterialUniformBuffer,
    ) {
        let Some(tilemap_uniform) = uniform_buffers.binding() else {
            return;
        };

        let Some(material_uniform) = std_material_uniform_buffer.binding() else {
            return;
        };

        self.tilemap_uniform_buffer = Some(render_device.create_bind_group(
            Some("tilemap_uniform_buffers_bind_group"),
            &entitiles_pipeline.uniform_buffers_layout,
            &BindGroupEntries::sequential((tilemap_uniform, material_uniform)),
        ));
    }

    pub fn bind_storage_buffers(
        &mut self,
        render_device: &RenderDevice,
        storage_buffers: &mut TilemapStorageBuffers,
        entitiles_pipeline: &EntiTilesPipeline,
    ) {
        storage_buffers
            .bindings()
            .into_iter()
            .for_each(|(tilemap, resource)| {
                self.storage_buffers.insert(
                    tilemap,
                    render_device.create_bind_group(
                        Some("tilemap_storage_bind_group"),
                        &entitiles_pipeline.storage_buffers_layout,
                        &BindGroupEntries::single(resource),
                    ),
                );
            });
    }

    pub fn prepare_materials(
        &mut self,
        material: &AssetId<StandardTilemapMaterial>,
        render_device: &RenderDevice,
        textures_storage: &TilemapTexturesStorage,
        entitiles_pipeline: &EntiTilesPipeline,
    ) -> bool {
        let Some(texture) = textures_storage.get_texture(material) else {
            return false;
        };

        if !self.materials.contains_key(material) {
            self.materials.insert(
                *material,
                render_device.create_bind_group(
                    Some("color_texture_bind_group"),
                    &entitiles_pipeline.color_texture_layout,
                    &BindGroupEntries::sequential((&texture.texture_view, &texture.sampler)),
                ),
            );
        }

        true
    }
}

#[derive(Resource)]
pub struct TilemapBindGroupLayouts {
    pub view_layout: BindGroupLayout,
    pub tilemap_uniforms_layout: BindGroupLayout,
    pub tilemap_storage_layout: BindGroupLayout,
    pub color_texture_layout: BindGroupLayout,
}

impl FromWorld for TilemapBindGroupLayouts {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            "tilemap_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                binding::uniform_buffer::<ViewUniform>(true),
            ),
        );

        let tilemap_uniforms_layout = render_device.create_bind_group_layout(
            "tilemap_uniforms_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    binding::uniform_buffer::<TilemapUniform>(true),
                    binding::uniform_buffer::<StandardMaterialUniform>(true),
                ),
            ),
        );

        let tilemap_storage_layout = render_device.create_bind_group_layout(
            "tilemap_storage_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                binding::storage_buffer_read_only::<i32>(false),
            ),
        );

        #[cfg(not(feature = "atlas"))]
        let color_texture_layout = render_device.create_bind_group_layout(
            "color_texture_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    binding::texture_2d_array(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        #[cfg(feature = "atlas")]
        let color_texture_layout = render_device.create_bind_group_layout(
            "color_texture_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    binding::texture_2d(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        Self {
            view_layout,
            tilemap_uniforms_layout,
            tilemap_storage_layout,
            color_texture_layout,
        }
    }
}
