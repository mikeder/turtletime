// Fills the shape of a sprite with a single color.
// Drawn on top of a player to make it flash when it takes damage.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BlinkMaterial {
    // the part of the texture to draw: min.xy and max.xy in uv coordinates
    uv_rect: vec4<f32>,
    // the color to flash, alpha is how strong the flash currently is
    color: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: BlinkMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mix(material.uv_rect.xy, material.uv_rect.zw, mesh.uv);
    // only the shape of the sprite is used, its colors are replaced
    let shape = textureSample(texture, texture_sampler, uv).a;
    return vec4<f32>(material.color.rgb, material.color.a * shape);
}
