// ========================================================================
// 6D Quantum SLAM Compute Shader
// ========================================================================

// ------------------------------------------------------------------------
// Data Structures
// ------------------------------------------------------------------------

struct Uniforms {
    resolution: vec2<f32>,   // 画面解像度 (x, y)
    time: f32,               // 経過時間 t
    wave_number: f32,        // 波数 k (不確定性の逆数)
    decay_factor: f32,       // 距離減衰率
    feedback_strength: f32,  // 時間フィードバック強度 (0.0 ~ 1.0)
    num_landmarks: u32,      // ランドマーク数
    camera_pos: vec2<f32>,   // (デバッグ用) 真のカメラ位置
};

struct Landmark {
    position: vec2<f32>,     // ランドマークの空間位置
    observed_dist: f32,      // カメラから観測された距離
    confidence: f32,         // 信頼度 (量子の振幅に対応)
    phase_offset: f32,       // 時間的位相ズレ
};

// ------------------------------------------------------------------------
// Bindings
// ------------------------------------------------------------------------

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> landmarks: array<Landmark>;

// Ping-Pong Buffering for Temporal Feedback
// Texture A: 前フレームの結果 (読み込み用)
// Texture B: 今回の書き込み先
@group(0) @binding(2) var prev_frame_texture: texture_2d<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba8unorm, write>;

// ------------------------------------------------------------------------
// Math Helpers (Complex Numbers)
// ------------------------------------------------------------------------

fn complex_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return a + b;
}

fn complex_mul_scalar(a: vec2<f32>, s: f32) -> vec2<f32> {
    return a * s;
}

// 複素指数関数: e^{i * theta} = cos(theta) + i*sin(theta)
fn complex_exp(theta: f32) -> vec2<f32> {
    return vec2<f32>(cos(theta), sin(theta));
}

// 確率密度: |z|^2
fn probability_density(z: vec2<f32>) -> f32 {
    return z.x * z.x + z.y * z.y;
}

// ------------------------------------------------------------------------
// Main Kernel
// ------------------------------------------------------------------------

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = u32(uniforms.resolution.x);
    let height = u32(uniforms.resolution.y);

    if (global_id.x >= width || global_id.y >= height) {
        return;
    }

    // ピクセル座標を UV空間 (-1.0 ~ 1.0) に正規化
    // アスペクト比を維持
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv = (vec2<f32>(global_id.xy) / uniforms.resolution) * 2.0 - 1.0;
    let pos_space = vec2<f32>(uv.x * aspect, uv.y);

    // ------------------------------------------------------------
    // Step 1: 波動関数の重ね合わせ (Quantum Superposition)
    // ------------------------------------------------------------
    // 全てのランドマークからの「逆観測波」を複素加算する
    var psi: vec2<f32> = vec2<f32>(0.0, 0.0);

    for (var i = 0u; i < uniforms.num_landmarks; i = i + 1u) {
        let lm = landmarks[i];

        // 仮説: もしカメラが「ここ(pos_space)」にいるとしたら、距離は？
        let hypo_dist = distance(pos_space, lm.position);

        // 残差 (Residual): 仮説距離 - 観測距離
        // これが 0 に近い場所ほど、位相が揃う (Constructive Interference)
        let residual = hypo_dist - lm.observed_dist;

        // 位相計算:
        // k * residual + temporal_phase
        // 時間項を入れることで「ゆらぎ」や「6次元的な回転」を表現
        let phase = uniforms.wave_number * residual + lm.phase_offset;

        // 振幅計算:
        // 距離が離れるほど不確かさが増す (減衰)
        let amplitude = lm.confidence * exp(-uniforms.decay_factor * abs(residual));

        // 波動関数への寄与
        let wave = complex_mul_scalar(complex_exp(phase), amplitude);
        psi = complex_add(psi, wave);
    }

    // ------------------------------------------------------------
    // Step 2: 確率密度の収縮 (Wavefunction Collapse)
    // ------------------------------------------------------------
    // 現在のフレームにおける瞬間的な存在確率
    let current_prob = probability_density(psi);

    // ------------------------------------------------------------
    // Step 3: 時間的フィードバック (Tenet Feedback)
    // ------------------------------------------------------------
    // 前のフレームの確率場をサンプリング
    // textureLoad は整数座標(ivec2)を使う
    let prev_color = textureLoad(prev_frame_texture, vec2<i32>(global_id.xy), 0);
    // 直前の確率は Gチャンネル に入っていると仮定 (Sci-Fi Green)
    let prev_prob = prev_color.g; 

    // 過去と現在の融合 (Incoherent mixing)
    // alpha = feedback_strength
    // これにより、確率の「軌跡」が描かれ、過去の情報が現在を拘束する
    let mixed_prob = mix(current_prob, prev_prob, uniforms.feedback_strength);

    // ------------------------------------------------------------
    // Step 4: 可視化レンダリング
    // ------------------------------------------------------------
    // 確率が高いほど明るく発光
    // 真の解周辺は強め合い(干渉)、誤った解は弱め合う
    
    // R: 瞬間的な位相の干渉 (赤くチラつくノイズ成分)
    let r = current_prob * 0.1;
    
    // G: 時間積分された確かな存在確率 (量子SLAMの解)
    let g = mixed_prob * 2.0; 
    
    // B: ランドマーク近傍のポテンシャル可視化
    let b = mixed_prob * 0.5 + 0.1 * sin(uniforms.time * 2.0);

    // 真のカメラ位置を表示（デバッグ用：白い点）
    let dist_to_cam = distance(pos_space, uniforms.camera_pos);
    let cam_marker = 1.0 - smoothstep(0.02, 0.03, dist_to_cam);

    let final_color = vec4<f32>(
        r + cam_marker, 
        g + cam_marker, 
        b + cam_marker, 
        1.0
    );

    textureStore(output_texture, global_id.xy, final_color);
}