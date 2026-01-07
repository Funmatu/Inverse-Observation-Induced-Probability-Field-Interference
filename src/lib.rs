// Rust 1.78+ と PyO3 0.20 の互換性警告をプロジェクトレベルで抑制
#![allow(non_local_definitions)]

use serde::{Serialize, Deserialize};
use bytemuck::{Pod, Zeroable};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// ============================================================================
//  Shared Data Structures (CPU/GPU Common)
// ============================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
pub struct Landmark {
    pub position: [f32; 2],
    pub observed_dist: f32,
    pub confidence: f32,
    pub phase_offset: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub wave_number: f32,
    pub decay_factor: f32,
    pub feedback_strength: f32,
    pub num_landmarks: u32,
    pub _pad: u32, // WGSLのアライメント(8byte)調整用パディング
    pub camera_pos: [f32; 2],
}

// ============================================================================
//  1. Physics Core (Pure Rust - CPU Implementation)
// ============================================================================

pub struct QuantumSlamCore {
    pub landmarks: Vec<Landmark>,
    pub wave_number: f64,
}

impl QuantumSlamCore {
    pub fn new(wave_number: f64) -> Self {
        Self {
            landmarks: Vec::new(),
            wave_number,
        }
    }

    pub fn add_landmark(&mut self, x: f32, y: f32) {
        self.landmarks.push(Landmark {
            position: [x, y],
            observed_dist: 0.0, // Init
            confidence: 1.0,
            phase_offset: 0.0,
        });
    }

    pub fn observe(&mut self, true_cam_x: f32, true_cam_y: f32) {
        for lm in &mut self.landmarks {
            let dx = lm.position[0] - true_cam_x;
            let dy = lm.position[1] - true_cam_y;
            lm.observed_dist = (dx * dx + dy * dy).sqrt();
        }
    }

    pub fn probability_at(&self, x: f32, y: f32) -> f64 {
        let mut re_sum = 0.0;
        let mut im_sum = 0.0;

        for lm in &self.landmarks {
            let dx = x - lm.position[0];
            let dy = y - lm.position[1];
            let hypo_dist = (dx * dx + dy * dy).sqrt();
            
            let residual = hypo_dist - lm.observed_dist;
            let phase = self.wave_number as f32 * residual;
            let amp = lm.confidence * (-2.0 * residual.abs()).exp();

            re_sum += amp * phase.cos();
            im_sum += amp * phase.sin();
        }

        (re_sum * re_sum + im_sum * im_sum) as f64
    }
}

// ============================================================================
//  2. Python Bindings (PyO3)
// ============================================================================
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyclass]
pub struct PyQuantumSlam {
    core: QuantumSlamCore,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyQuantumSlam {
    #[new]
    fn new(wave_number: f64) -> Self {
        Self { core: QuantumSlamCore::new(wave_number) }
    }

    fn add_landmark(&mut self, x: f32, y: f32) {
        self.core.add_landmark(x, y);
    }

    fn update_observation(&mut self, cam_x: f32, cam_y: f32) {
        self.core.observe(cam_x, cam_y);
    }

    fn get_probability(&self, x: f32, y: f32) -> f64 {
        self.core.probability_at(x, y)
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn inverse_observation_induced_probability_field_interference(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyQuantumSlam>()?;
    Ok(())
}

// ============================================================================
//  3. WGPU Renderer (WASM / Visualization)
// ============================================================================

#[cfg(feature = "wasm")]
const SHADER_SOURCE: &str = include_str!("shader.wgsl");

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct QuantumRenderer {
    #[wasm_bindgen(skip)]
    pub device: wgpu::Device,
    #[wasm_bindgen(skip)]
    pub queue: wgpu::Queue,
    #[wasm_bindgen(skip)]
    pub surface: wgpu::Surface<'static>,
    #[wasm_bindgen(skip)]
    pub config: wgpu::SurfaceConfiguration,
    #[wasm_bindgen(skip)]
    pub pipeline: wgpu::ComputePipeline,
    #[wasm_bindgen(skip)]
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    // Double Buffering
    #[wasm_bindgen(skip)]
    pub texture_a: wgpu::Texture,
    #[wasm_bindgen(skip)]
    pub texture_a_view: wgpu::TextureView,
    #[wasm_bindgen(skip)]
    pub texture_b: wgpu::Texture,
    #[wasm_bindgen(skip)]
    pub texture_b_view: wgpu::TextureView,
    
    #[wasm_bindgen(skip)]
    pub uniform_buffer: wgpu::Buffer,
    #[wasm_bindgen(skip)]
    pub landmark_buffer: wgpu::Buffer,
    
    start_time: f64,
    frame_count: u64,
    
    landmarks: Vec<Landmark>,
    camera_pos: [f32; 2],
    
    width: u32,
    height: u32,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl QuantumRenderer {
    pub async fn new(canvas_id: &str) -> Result<QuantumRenderer, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        
        let width = canvas.width();
        let height = canvas.height();

        let instance = wgpu::Instance::default();
        
        let surface_target = wgpu::SurfaceTarget::Canvas(canvas);
        let surface = instance.create_surface(surface_target).map_err(|e| e.to_string())?;
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or("No adapter found")?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Quantum Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
        }, None).await.map_err(|e| e.to_string())?;

        let surface_caps = surface.get_capabilities(&adapter);
        // let surface_format = surface_caps.formats.iter()
        //     .copied()
        //     .find(|f: &wgpu::TextureFormat| f.is_srgb())
        //     .unwrap_or(surface_caps.formats[0]);
        
        // サーフェスのフォーマット選択ロジックを変更
        // 以前のロジック: sRGBを探す -> BGRAが選ばれることが多い
        // 今回のロジック: Compute Shaderの出力(Rgba8Unorm)に合わせて、サーフェスもRgba8Unormを強制する
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Rgba8Unorm)
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Ping-Pong Textures
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Probability Field"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING 
                 | wgpu::TextureUsages::STORAGE_BINDING 
                 | wgpu::TextureUsages::COPY_SRC 
                 | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let texture_a = device.create_texture(&texture_desc);
        let texture_b = device.create_texture(&texture_desc);
        let texture_a_view = texture_a.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_b_view = texture_b.create_view(&wgpu::TextureViewDescriptor::default());

        // Buffers
        let uniform_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let landmark_size = (std::mem::size_of::<Landmark>() * 100) as wgpu::BufferAddress;
        let landmark_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Landmark Buffer"),
            size: landmark_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quantum Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER_SOURCE)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                },
                // Landmarks
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                },
                // Input Texture (Prev Frame)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                // Output Texture (Current Frame)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { 
                        access: wgpu::StorageTextureAccess::WriteOnly, 
                        format: wgpu::TextureFormat::Rgba8Unorm, 
                        view_dimension: wgpu::TextureViewDimension::D2 
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(), 
            cache: None,
        });

        let landmarks = vec![
            Landmark { position: [0.0, 0.5], observed_dist: 0.0, confidence: 1.0, phase_offset: 0.0 },
            Landmark { position: [0.5, -0.5], observed_dist: 0.0, confidence: 1.0, phase_offset: 0.0 },
            Landmark { position: [-0.5, -0.5], observed_dist: 0.0, confidence: 1.0, phase_offset: 0.0 },
        ];

        Ok(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            bind_group_layout,
            texture_a,
            texture_a_view,
            texture_b,
            texture_b_view,
            uniform_buffer,
            landmark_buffer,
            start_time: js_sys::Date::now(),
            frame_count: 0,
            landmarks,
            camera_pos: [0.0, 0.0],
            width,
            height,
        })
    }

    pub fn update(&mut self) {
        let now = js_sys::Date::now();
        let t = (now - self.start_time) / 1000.0;
        
        self.camera_pos = [
            (t * 0.5).sin() as f32 * 0.5,
            (t * 0.3).cos() as f32 * 0.5
        ];

        for lm in &mut self.landmarks {
            let dx = lm.position[0] - self.camera_pos[0];
            let dy = lm.position[1] - self.camera_pos[1];
            lm.observed_dist = (dx*dx + dy*dy).sqrt();
            lm.phase_offset = (t as f32 * 2.0).sin() * 0.5;
        }

        self.queue.write_buffer(&self.landmark_buffer, 0, bytemuck::cast_slice(&self.landmarks));

        let uniforms = Uniforms {
            resolution: [self.width as f32, self.height as f32],
            time: t as f32,
            wave_number: 80.0,
            decay_factor: 5.0,
            feedback_strength: 0.90,
            num_landmarks: self.landmarks.len() as u32,
            _pad: 0,
            camera_pos: self.camera_pos,
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render(&mut self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let (input_view, output_view, source_tex) = if self.frame_count % 2 == 0 {
            (&self.texture_a_view, &self.texture_b_view, &self.texture_b)
        } else {
            (&self.texture_b_view, &self.texture_a_view, &self.texture_a)
        };

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Frame BindGroup"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.landmark_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(input_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(output_view) },
            ],
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups((self.width + 15) / 16, (self.height + 15) / 16, 1);
        }

        if let Some(surface_texture) = self.get_current_texture() {
            let _surface_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture { texture: source_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::ImageCopyTexture { texture: &surface_texture.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 }
            );

            self.queue.submit(Some(encoder.finish()));
            surface_texture.present();
        } else {
            self.queue.submit(Some(encoder.finish()));
        }

        self.frame_count += 1;
    }

    fn get_current_texture(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(texture) => Some(texture),
            Err(wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                None
            },
            Err(_) => None,
        }
    }
}