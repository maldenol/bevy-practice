import bevy_pbr::render::forward_io::VertexOutput;

const SPACE_GRID_PERIOD = 50.0;
const SPACE_GRID_SIZE = 0.5;
const SPACE_GRID_ALPHA = 0.005;

struct CameraSettings {
    pos: vec3<f32>,
    look_at: vec3<f32>,
}

struct GravitySrc {
    pos: vec3<f32>,
    vel: vec3<f32>,
    mass: f32,
    index: u32,
    central_body_index: u32,
}

struct GravitySrcBuffer {
    num: u32,
    buffer: array<GravitySrc, 32>,
    max_mass: f32,
    max_mass_radius: f32,
    gravity_constant: f32,
    lagrange_threshold: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> grid_mode: u32;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var<uniform> camera_settings: CameraSettings;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var<uniform> gravity_src_buffer: GravitySrcBuffer;

fn space(
    world_pos: vec2<f32>,
) -> vec4<f32> {
    let camera_dist = distance(camera_settings.pos, camera_settings.look_at);

    if (true
        && round(abs(world_pos.x) / SPACE_GRID_SIZE) % (SPACE_GRID_PERIOD / SPACE_GRID_SIZE) != 0
        && round(abs(world_pos.y) / SPACE_GRID_SIZE) % (SPACE_GRID_PERIOD / SPACE_GRID_SIZE) != 0
    ) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    let look_at_distance = distance(camera_settings.look_at.xy, world_pos.xy);
    let alpha = SPACE_GRID_ALPHA * min(1.0 - (look_at_distance - camera_dist) / (camera_dist * 0.5), 1.0);

    if abs(world_pos.x) < 0.5 {
        return vec4(0.0, 1.0, 0.0, alpha);
    } else if abs(world_pos.y) < 0.5 {
        return vec4(1.0, 0.0, 0.0, alpha);
    }

    return vec4(1.0, 1.0, 1.0, alpha);
}

fn potential(
    world_pos: vec3<f32>,
) -> vec4<f32> {
    var pot_vec: vec3<f32> = vec3(0.0, 0.0, 0.0);
    for (var i: u32 = 0; i < gravity_src_buffer.num; i++) {
        let dist = distance(gravity_src_buffer.buffer[i].pos, world_pos);
        let scalar = gravity_src_buffer.buffer[i].mass / dist;
        let dir = normalize(gravity_src_buffer.buffer[i].pos - world_pos);
        pot_vec += scalar * dir;
    }

    let max_pot = gravity_src_buffer.max_mass / gravity_src_buffer.max_mass_radius;
    let pot = length(pot_vec) / max_pot;

    return vec4(pot, pot, pot, 1.0);
}

fn acceleration(
    world_pos: vec3<f32>,
) -> vec4<f32> {
    var accel_vec: vec3<f32> = vec3(0.0, 0.0, 0.0);
    for (var i: u32 = 0; i < gravity_src_buffer.num; i++) {
        let dist = distance(gravity_src_buffer.buffer[i].pos, world_pos);
        let scalar = gravity_src_buffer.buffer[i].mass / dist / dist;
        let dir = normalize(gravity_src_buffer.buffer[i].pos - world_pos);
        accel_vec += scalar * dir;
    }

    let max_accel = gravity_src_buffer.max_mass / gravity_src_buffer.max_mass_radius / gravity_src_buffer.max_mass_radius;
    let accel = length(accel_vec) / max_accel;

    return vec4(accel, accel, accel, 1.0);
}

fn lagrange(
    world_pos: vec3<f32>,
) -> vec4<f32> {
    let gc = gravity_src_buffer.gravity_constant;

    for (var i: u32 = 0; i < gravity_src_buffer.num; i++) {
        for (var j: u32 = 0; j < gravity_src_buffer.num; j++) {
            let i1 = gravity_src_buffer.buffer[i].index;
            let cbi2 = gravity_src_buffer.buffer[j].central_body_index;
            if cbi2 != i1 {
                continue;
            }

            let m1 = gravity_src_buffer.buffer[i].mass;
            let m2 = gravity_src_buffer.buffer[j].mass;

            let p1 = gravity_src_buffer.buffer[i].pos;
            let p2 = gravity_src_buffer.buffer[j].pos;
            let p0 = p1 * m1 / (m1 + m2) + p2 * m2 / (m1 + m2);
            let p3 = world_pos;

            let r31 = distance(p1, p3);
            let r32 = distance(p2, p3);
            let r03 = distance(p3, p0);
            let r02 = distance(p2, p0);

            let v1 = gravity_src_buffer.buffer[i].vel;
            let v2 = gravity_src_buffer.buffer[j].vel;
            let v0 = (v1 * m1 + v2 * m2) / (m1 + m2);
            let v02 = length(v2 - v0);
            let v03 = v02 * r03 / r02;

            let g31 = normalize(p1 - p3) * gc * m1 / r31 / r31;
            let g32 = normalize(p2 - p3) * gc * m2 / r32 / r32;
            let a03 = normalize(p3 - p0) * v03 * v03 / r03;
            let total_accel = g31 + g32 + a03;

            if length(total_accel) < gravity_src_buffer.lagrange_threshold * gc {
                return vec4(1.0, 1.0, 1.0, 1.0);
            }
        }
    }

    return vec4(0.0, 0.0, 0.0, 0.0);
}

@fragment
fn fragment(
    frag: VertexOutput,
) -> @location(0) vec4<f32> {
    switch grid_mode {
        case 1: {
            return space(frag.world_position.xy);
        }
        case 2: {
            return potential(frag.world_position.xyz);
        }
        case 3: {
            return acceleration(frag.world_position.xyz);
        }
        case 4: {
            return lagrange(frag.world_position.xyz);
        }
        default: {
            return vec4(0.0, 0.0, 0.0, 0.0);
        }
    }
}
