//! Tiny debug text overlay (FPS counter), drawn on top of the finished frame in window pixels.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const ATLAS: &[u8] = include_bytes!("font_atlas.bin");
const ATLAS_W: u32 = 160;
const ATLAS_H: u32 = 126; // the last cell row is solid white (used for the backdrop)
const CELL_W: f32 = 10.0;
const CELL_H: f32 = 18.0;
const COLS: u32 = 16;
const MAX_CHARS: usize = 512;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct V {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

const SHADER: &str = r#"
struct U { size: vec2<f32>, pad: vec2<f32> };
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var t: texture_2d<f32>;
@group(0) @binding(2) var s: sampler;
struct VO { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) c: vec4<f32> };
@vertex fn vs(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) c: vec4<f32>) -> VO {
    var o: VO;
    o.p = vec4<f32>(pos.x / u.size.x * 2.0 - 1.0, 1.0 - pos.y / u.size.y * 2.0, 0.0, 1.0);
    o.uv = uv;
    o.c = c;
    return o;
}
@fragment fn fs(i: VO) -> @location(0) vec4<f32> {
    let a = textureSample(t, s, i.uv).r;
    return vec4<f32>(i.c.rgb, i.c.a * a);
}
"#;

pub struct Overlay {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Overlay {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Overlay {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font"),
            size: wgpu::Extent3d { width: ATLAS_W, height: ATLAS_H, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            ATLAS,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(ATLAS_W), rows_per_image: Some(ATLAS_H) },
            wgpu::Extent3d { width: ATLAS_W, height: ATLAS_H, depth_or_array_layers: 1 },
        );
        let view = tex.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("overlay"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("overlay"), bind_group_layouts: &[&layout], push_constant_ranges: &[] });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs",
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<V>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        Overlay { pipeline, layout, view, sampler }
    }

    /// Draw `text` (lines separated by newlines) in the top-left corner of a `size` pixel target.
    pub fn draw(&self, device: &wgpu::Device, enc: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, size: (u32, u32), text: &str) {
        let scale = (size.1 as f32 / 720.0).max(1.0);
        let mut verts: Vec<V> = Vec::new();
        let (x0, y0) = (8.0 * scale, 6.0 * scale);
        let (mut x, mut y) = (x0, y0);
        let mut n = 0;
        let quad = |verts: &mut Vec<V>, px: f32, py: f32, w: f32, h: f32, uv: [f32; 4], c: [f32; 4]| {
            let (u0, v0, u1, v1) = (uv[0], uv[1], uv[2], uv[3]);
            for (dx, dy, u, v) in [(0.0, 0.0, u0, v0), (w, 0.0, u1, v0), (0.0, h, u0, v1), (w, 0.0, u1, v0), (w, h, u1, v1), (0.0, h, u0, v1)] {
                verts.push(V { pos: [px + dx, py + dy], uv: [u, v], color: c });
            }
        };
        // dark backdrop behind the whole block so the text stays readable over the game
        {
            let (mut cols, mut rows, mut cur) = (0usize, 1usize, 0usize);
            for ch in text.chars() {
                if ch == '\n' {
                    rows += 1;
                    cur = 0;
                } else {
                    cur += 1;
                    cols = cols.max(cur);
                }
            }
            let solid = [2.0 / ATLAS_W as f32, 110.0 / ATLAS_H as f32, 8.0 / ATLAS_W as f32, 124.0 / ATLAS_H as f32];
            let pad = 6.0 * scale;
            quad(&mut verts, x0 - pad, y0 - pad * 0.5, cols as f32 * CELL_W * scale + 2.0 * pad, rows as f32 * CELL_H * scale + pad, solid, [0.0, 0.0, 0.0, 0.55]);
        }
        for ch in text.chars() {
            if n >= MAX_CHARS {
                break;
            }
            if ch == '\n' {
                x = x0;
                y += CELL_H * scale;
                continue;
            }
            let c = (ch as u32).clamp(32, 126) - 32;
            let (cx, cy) = ((c % COLS) as f32 * CELL_W, (c / COLS) as f32 * CELL_H);
            let uv = [cx / ATLAS_W as f32, cy / ATLAS_H as f32, (cx + CELL_W) / ATLAS_W as f32, (cy + CELL_H) / ATLAS_H as f32];
            // drop shadow first, then the glyph
            quad(&mut verts, x + scale, y + scale, CELL_W * scale, CELL_H * scale, uv, [0.0, 0.0, 0.0, 0.85]);
            quad(&mut verts, x, y, CELL_W * scale, CELL_H * scale, uv, [1.0, 1.0, 0.2, 1.0]);
            x += CELL_W * scale;
            n += 1;
        }
        if verts.is_empty() {
            return;
        }
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("overlay verts"), contents: bytemuck::cast_slice(&verts), usage: wgpu::BufferUsages::VERTEX });
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("overlay size"),
            contents: bytemuck::cast_slice(&[size.0 as f32, size.1 as f32, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&self.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("overlay"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.set_vertex_buffer(0, vbuf.slice(..));
        pass.draw(0..verts.len() as u32, 0..1);
    }
}
