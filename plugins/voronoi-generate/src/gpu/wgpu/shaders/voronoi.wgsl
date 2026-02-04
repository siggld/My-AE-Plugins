struct Params {
    size: vec4<u32>,
    seed: vec4<u32>,
    cell: vec4<f32>,
    extra: vec4<f32>,
    misc: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> out_buf: array<vec4<f32>>;

struct Site {
    x: f32,
    y: f32,
    w: f32,
    hash: u32,
};

fn hash_u32(mut x: u32) -> u32 {
    x = x ^ (x >> 16u);
    x = x * 0x7FEB352Du;
    x = x ^ (x >> 15u);
    x = x * 0x846CA68Bu;
    x = x ^ (x >> 16u);
    return x;
}

fn hash3(x: i32, y: i32, w: i32, seed: u32) -> u32 {
    let ux = bitcast<u32>(x);
    let uy = bitcast<u32>(y);
    let uw = bitcast<u32>(w);
    var h = seed ^ 0x9E3779B9u;
    h = h + ux * 0x85EBCA6Bu;
    h = h + uy * 0xC2B2AE35u;
    h = h + uw * 0x27D4EB2Du;
    return hash_u32(h);
}

fn rand01(h: u32) -> f32 {
    return f32(h) / 4294967295.0;
}

fn cell_point(cell_x: i32, cell_y: i32, cell_w: i32, randomness: f32, seed: u32) -> Site {
    let h = hash3(cell_x, cell_y, cell_w, seed);
    let rx = rand01(hash_u32(h ^ 0xA511E9B3u));
    let ry = rand01(hash_u32(h ^ 0x63D83595u));
    let rw = rand01(hash_u32(h ^ 0x1F1D8E33u));
    let ox = 0.5 + (rx - 0.5) * randomness;
    let oy = 0.5 + (ry - 0.5) * randomness;
    let ow = 0.5 + (rw - 0.5) * randomness;
    return Site(f32(cell_x) + ox, f32(cell_y) + oy, f32(cell_w) + ow, h);
}

fn hash_color(h: u32) -> vec3<f32> {
    let r = rand01(hash_u32(h ^ 0xB5297A4Du));
    let g = rand01(hash_u32(h ^ 0x68E31DA4u));
    let b = rand01(hash_u32(h ^ 0x1B56C4E9u));
    return vec3<f32>(r, g, b);
}

fn metric_distance(dx: f32, dy: f32, dw: f32, metric: u32, lp_exp: f32) -> f32 {
    let adx = abs(dx);
    let ady = abs(dy);
    let adw = abs(dw);
    if metric == 0u {
        return sqrt(dx * dx + dy * dy + dw * dw);
    }
    if metric == 1u {
        return adx + ady + adw;
    }
    if metric == 2u {
        return max(max(adx, ady), adw);
    }
    let p = max(lp_exp, 0.1);
    let s = pow(adx, p) + pow(ady, p) + pow(adw, p);
    return pow(s, 1.0 / p);
}

fn smoothstep01(x: f32) -> f32 {
    let t = clamp(x, 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn smooth_blend(d1: f32, d2: f32, smoothness: f32) -> f32 {
    if smoothness <= 0.0 {
        return 0.0;
    }
    let t = clamp((d2 - d1) / smoothness, 0.0, 1.0);
    return 0.5 * (1.0 - smoothstep01(t));
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + (b - a) * t;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let out_w = params.size.x;
    let out_h = params.size.y;
    if (gid.x >= out_w || gid.y >= out_h) {
        return;
    }

    let inv_cell_x = params.cell.x;
    let inv_cell_y = params.cell.y;
    let randomness = params.cell.z;
    let lp_exp = params.cell.w;
    let inv_cell_w = params.extra.x;
    let smoothness = params.misc.x;
    let w_value = params.misc.y;
    let offset_x = params.misc.z;
    let offset_y = params.misc.w;

    let px = (f32(gid.x) + 0.5 - offset_x) * inv_cell_x;
    let py = (f32(gid.y) + 0.5 - offset_y) * inv_cell_y;
    let pw = w_value * inv_cell_w;
    let cell_x = i32(floor(px));
    let cell_y = i32(floor(py));
    let cell_w = i32(floor(pw));

    var d1 = 1e20;
    var d2 = 1e20;
    var nearest = Site(0.0, 0.0, 0.0, 0u);
    var second = Site(0.0, 0.0, 0.0, 0u);

    for (var nw: i32 = cell_w - 1; nw <= cell_w + 1; nw = nw + 1) {
        for (var ny: i32 = cell_y - 1; ny <= cell_y + 1; ny = ny + 1) {
            for (var nx: i32 = cell_x - 1; nx <= cell_x + 1; nx = nx + 1) {
                let site = cell_point(nx, ny, nw, randomness, params.seed.x);
                let dx = px - site.x;
                let dy = py - site.y;
                let dw = pw - site.w;
                let d = metric_distance(dx, dy, dw, params.size.z, lp_exp);
                if (d < d1) {
                    d2 = d1;
                    second = nearest;
                    d1 = d;
                    nearest = site;
                } else if (d < d2) {
                    d2 = d;
                    second = site;
                }
            }
        }
    }

    if (d2 < d1) {
        let tmp = d1;
        d1 = d2;
        d2 = tmp;
        let tmp_site = nearest;
        nearest = second;
        second = tmp_site;
    }

    let blend = smooth_blend(d1, d2, smoothness);
    var out = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (params.size.w == 0u) {
        let c1 = hash_color(nearest.hash);
        let c2 = hash_color(second.hash);
        out = vec4<f32>(vec3<f32>(
            lerp(c1.x, c2.x, blend),
            lerp(c1.y, c2.y, blend),
            lerp(c1.z, c2.z, blend)
        ), 1.0);
    } else if (params.size.w == 1u) {
        let grid_w = max(f32(out_w) * inv_cell_x, 1e-6);
        let grid_h = max(f32(out_h) * inv_cell_y, 1e-6);
        let r = nearest.x / grid_w;
        let g = nearest.y / grid_h;
        out = vec4<f32>(r, g, 0.0, 1.0);
    } else if (params.size.w == 2u) {
        let v = lerp(d1, d2, blend);
        out = vec4<f32>(v, v, v, 1.0);
    } else if (params.size.w == 3u) {
        let v = d1;
        out = vec4<f32>(v, v, v, 1.0);
    } else {
        let v = max(d2 - d1, 0.0);
        out = vec4<f32>(v, v, v, 1.0);
    }

    let idx = gid.y * out_w + gid.x;
    out_buf[idx] = out;
}
