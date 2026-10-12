// use bevy::{
//     material::{descriptor::RenderPipelineDescriptor, specialize::SpecializedMeshPipelineError},
//     mesh::MeshVertexBufferLayoutRef,
//     pbr::{MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
//     prelude::*,
//     render::render_resource::{AsBindGroup, ShaderType},
//     shader::ShaderDefVal,
// };
//
// #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
// pub struct GridMaterialVoxel {
//     #[uniform(75)]
//     pub uniforms: GridMaterialUniforms,
// }
//
// #[derive(Clone, Debug, ShaderType, Default)]
// pub struct GridMaterialUniforms {
//     pub grid_scale: f32,
//     pub line_width: f32,
// }
//
// impl MaterialExtension for GridMaterialVoxel {
//     // fn vertex_shader() -> bevy::shader::ShaderRef {
//     //     VOXEL_TEXTURE_SHADER_HANDLE.into()
//     // }
//
//     fn fragment_shader() -> bevy::shader::ShaderRef {
//         "shaders/grid.wgsl".into()
//     }
//
//     // fn specialize(
//     //     _pipeline: &MaterialExtensionPipeline,
//     //     descriptor: &mut RenderPipelineDescriptor,
//     //     layout: &MeshVertexBufferLayoutRef,
//     //     _key: MaterialExtensionKey<Self>,
//     // ) -> Result<(), SpecializedMeshPipelineError> {
//     //     if descriptor
//     //         .vertex
//     //         .shader_defs
//     //         .contains(&ShaderDefVal::Bool("PREPASS_PIPELINE".into(), true))
//     //     {
//     //         return Ok(());
//     //     }
//     //
//     //     let voxel_layout = layout.0.get_layout(&vertex_layout())?;
//     //     if !descriptor.vertex.buffers.is_empty() {
//     //         descriptor.vertex.buffers[0] = voxel_layout;
//     //     } else {
//     //         descriptor.vertex.buffers = vec![voxel_layout];
//     //     }
//     //
//     //     Ok(())
//     // }
// }
