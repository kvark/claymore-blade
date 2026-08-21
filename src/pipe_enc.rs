//! Backend-specific PipelineEncoder lifetime arity.
//! Vulkan/Metal use two lifetimes; GLES/WebGL use one.

#[cfg(any(target_arch = "wasm32", gles))]
pub type PipeEnc<'a> = blade_graphics::PipelineEncoder<'a>;
#[cfg(not(any(target_arch = "wasm32", gles)))]
pub type PipeEnc<'a> = blade_graphics::PipelineEncoder<'a, 'a>;
