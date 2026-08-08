use crate::config::ParticleConfig;
use crate::error::Result;
use glam::Vec3;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub color: [f32; 4],
    pub life: f32,
    pub size: f32,
    pub _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleUniforms {
    pub time: f32,
    pub delta_time: f32,
    pub particle_count: u32,
    pub _pad: u32,
    pub center: [f32; 3],
    pub cohesion: f32,
    pub glow_intensity: f32,
    pub turbulence: f32,
    pub breath_phase: f32,
    pub expansion: f32,
}

pub struct ParticleEngine {
    _device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::RenderPipeline,
    particle_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    particles: Vec<Particle>,
    config: ParticleConfig,
    time: f32,
    breath_phase: f32,
    cohesion: f32,
    expansion: f32,
    center: Vec3,
}

impl ParticleEngine {
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: ParticleConfig,
    ) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("particle_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particle_uniforms"),
            contents: bytemuck::cast_slice(&[ParticleUniforms {
                time: 0.0,
                delta_time: 0.0,
                particle_count: config.min_particles as u32,
                _pad: 0,
                center: [0.0; 3],
                cohesion: 0.5,
                glow_intensity: config.glow_intensity,
                turbulence: config.turbulence,
                breath_phase: 0.0,
                expansion: 1.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("particle_uniform_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("particle_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("particle_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Particle>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 24,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 40,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32,
                        },
                        wgpu::VertexAttribute {
                            offset: 44,
                            shader_location: 4,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let particles = Self::create_particles(&config);

        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particle_buffer"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            _device: device,
            queue,
            pipeline,
            particle_buffer,
            uniform_buffer,
            uniform_bind_group,
            particles,
            config,
            time: 0.0,
            breath_phase: 0.0,
            cohesion: 0.5,
            expansion: 1.0,
            center: Vec3::ZERO,
        })
    }

    fn create_particles(config: &ParticleConfig) -> Vec<Particle> {
        let mut particles = Vec::with_capacity(config.max_particles);
        for _ in 0..config.min_particles {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let radius = rand::random::<f32>() * 100.0;
            let height = (rand::random::<f32>() - 0.5) * 200.0;

            particles.push(Particle {
                position: [angle.cos() * radius, height, angle.sin() * radius],
                velocity: [
                    (rand::random::<f32>() - 0.5) * config.base_speed,
                    (rand::random::<f32>() - 0.5) * config.base_speed,
                    (rand::random::<f32>() - 0.5) * config.base_speed,
                ],
                color: [
                    0.4 + rand::random::<f32>() * 0.2,
                    0.6 + rand::random::<f32>() * 0.2,
                    1.0,
                    0.8,
                ],
                life: rand::random::<f32>(),
                size: config.particle_size * (0.5 + rand::random::<f32>()),
                _pad: [0.0; 2],
            });
        }
        particles
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        center: Vec3,
        cohesion: f32,
        expansion: f32,
        glow: f32,
    ) {
        self.time += delta_time;
        self.breath_phase += delta_time * 0.8;
        self.cohesion = cohesion;
        self.expansion = expansion;
        self.center = center;

        let breath = self.breath_phase.sin() * self.config.turbulence;

        for particle in &mut self.particles {
            let to_center = center - Vec3::from(particle.position);
            let dist = to_center.length();

            let mut force = Vec3::ZERO;

            if dist > 0.1 {
                let attraction = to_center.normalize() * cohesion * 0.1;
                force += attraction;
            }

            force += Vec3::new(
                (self.time + particle.life).sin() * self.config.turbulence,
                (self.time * 0.7 + particle.life).cos() * self.config.turbulence,
                (self.time * 1.3 + particle.life).sin() * self.config.turbulence,
            );

            force.y += breath;

            let vel = Vec3::from(particle.velocity);
            let new_vel = (vel + force * delta_time) * 0.99;
            particle.velocity = new_vel.to_array();

            let pos = Vec3::from(particle.position);
            let new_pos = pos + new_vel * delta_time * expansion;
            particle.position = new_pos.to_array();

            particle.color[3] = 0.5 + glow * 0.5;
        }

        self.queue.write_buffer(
            &self.particle_buffer,
            0,
            bytemuck::cast_slice(&self.particles),
        );

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[ParticleUniforms {
                time: self.time,
                delta_time,
                particle_count: self.particles.len() as u32,
                _pad: 0,
                center: center.into(),
                cohesion,
                glow_intensity: glow,
                turbulence: self.config.turbulence,
                breath_phase: self.breath_phase,
                expansion,
            }]),
        );
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.particle_buffer.slice(..));
        render_pass.draw(0..6, 0..self.particles.len() as u32);
    }

    pub fn set_particle_count(&mut self, count: usize) {
        let count = count.min(self.config.max_particles);
        if count > self.particles.len() {
            let extra = count - self.particles.len();
            for _ in 0..extra {
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                let radius = rand::random::<f32>() * 100.0;
                let height = (rand::random::<f32>() - 0.5) * 200.0;

                self.particles.push(Particle {
                    position: [angle.cos() * radius, height, angle.sin() * radius],
                    velocity: [
                        (rand::random::<f32>() - 0.5) * self.config.base_speed,
                        (rand::random::<f32>() - 0.5) * self.config.base_speed,
                        (rand::random::<f32>() - 0.5) * self.config.base_speed,
                    ],
                    color: [
                        0.4 + rand::random::<f32>() * 0.2,
                        0.6 + rand::random::<f32>() * 0.2,
                        1.0,
                        0.8,
                    ],
                    life: rand::random::<f32>(),
                    size: self.config.particle_size * (0.5 + rand::random::<f32>()),
                    _pad: [0.0; 2],
                });
            }
        } else if count < self.particles.len() {
            self.particles.truncate(count);
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn config(&self) -> &ParticleConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let config = ParticleConfig::default();
        let particles = ParticleEngine::create_particles(&config);
        assert_eq!(particles.len(), config.min_particles);
    }

    #[test]
    fn test_particle_struct_size() {
        assert_eq!(std::mem::size_of::<Particle>(), 56);
    }

    #[test]
    fn test_particle_uniforms_size() {
        assert_eq!(std::mem::size_of::<ParticleUniforms>(), 48);
    }
}
