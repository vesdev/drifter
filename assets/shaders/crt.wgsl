#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct CrtSettings {
    bend: f32,
};

@group(0) @binding(2) var<uniform> settings: CrtSettings;

fn scanline(uv: vec2<f32>, screen: vec3<f32>) -> vec3<f32> {
    return screen.rgb - abs(sin(uv.y * 500.) * 2.) * 0.01;
}

fn curve(uv: vec2<f32>, bend: f32) -> vec2<f32> {
    var out = (uv - 0.5) * 2.2;

    out.x *= 1.0 + pow((abs(out.y) / bend), 2.0);
    out.y *= 1.0 + pow((abs(out.x) / bend), 2.0);
    out = out / 2.0 + 0.5;

    return out;
}

fn split(uv: vec2<f32>, tex: texture_2d<f32>) -> vec3<f32> {
    return vec3<f32>(
        textureSample(tex, samp, vec2<f32>(uv.x - 0.1 * 0.02, uv.y)).r, 
        textureSample(tex, samp, vec2<f32>(uv.x, uv.y)).g,
        textureSample(tex, samp, vec2<f32>(uv.x + 0.1 * 0.02, uv.y)).b
    ); 
}

@fragment
fn fragment(
    in: FullscreenVertexOutput
) -> @location(0) vec4<f32> {
    var uv = in.uv;
    
    let bent = curve(uv, settings.bend);

    var color = split(bent, screen_texture);
    color = scanline(bent, color);
    color *= 1.0 - f32(bent.x < 0.0 || bent.x > 1.0 || bent.y < 0.0 || bent.y > 1.0);
    return vec4<f32>(color.r, color.g, color.b, 1.0);
    // return textureSample(screen_texture, texture_sampler, in.uv) * vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
