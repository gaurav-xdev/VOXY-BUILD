struct Uniforms {
    time: f32,
    delta_time: f32,
    particle_count: u32,
    _pad: u32,
    center: vec3<f32>,
    cohesion: f32,
    glow_intensity: f32,
    turbulence: f32,
    breath_phase: f32,
    expansion: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) velocity: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) life: f32,
    @location(4) size: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) life: f32,
    @location(2) dist_from_center: f32,
};

struct VertexOutput_instanced {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) life: f32,
    @location(2) dist_from_center: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    particle: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let quad_positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    let quad_pos = quad_positions[vertex_index];

    let life_factor = particle.life;
    let breath = sin(uniforms.breath_phase) * 0.1;
    let effective_size = particle.size * (1.0 + breath) * uniforms.expansion;

    let world_pos = vec3<f32>(
        particle.position.x + quad_pos.x * effective_size,
        particle.position.y + quad_pos.y * effective_size,
        particle.position.z
    );

    let dist = length(particle.position - uniforms.center);

    out.clip_position = vec4<f32>(world_pos * 0.005, 1.0);
    out.color = vec4<f32>(
        particle.color.rgb * uniforms.glow_intensity,
        particle.color.a * life_factor
    );
    out.life = life_factor;
    out.dist_from_center = dist;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.dist_from_center) * 0.01;
    let alpha = in.color.a * exp(-dist * dist) * in.life;

    return vec4<f32>(in.color.rgb, alpha);
}
