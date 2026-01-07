import init, { QuantumRenderer } from './pkg/inverse_observation_induced_probability_field_interference.js';

async function run() {
    await init();
    
    const btn = document.getElementById('start-btn');
    const canvas = document.getElementById('quantum-canvas');
    
    // UI Elements
    const inputWave = document.getElementById('input-wave');
    const valWave = document.getElementById('val-wave');
    const inputFeedback = document.getElementById('input-feedback');
    const valFeedback = document.getElementById('val-feedback');

    // Resize canvas to full screen
    function resize() {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
    }
    window.addEventListener('resize', resize);
    resize();
    
    btn.addEventListener('click', async () => {
        btn.disabled = true;
        btn.innerText = "Running Simulation...";
        
        try {
            // Rust側のWGPU初期化
            const renderer = await QuantumRenderer.new("quantum-canvas");
            
            // Link Sliders to Rust Renderer
            inputWave.addEventListener('input', (e) => {
                const val = parseFloat(e.target.value);
                valWave.innerText = val.toFixed(1);
                renderer.set_wave_number(val);
            });

            inputFeedback.addEventListener('input', (e) => {
                const val = parseFloat(e.target.value);
                valFeedback.innerText = val.toFixed(2);
                renderer.set_feedback_strength(val);
            });

            // Initial Sync
            renderer.set_wave_number(parseFloat(inputWave.value));
            renderer.set_feedback_strength(parseFloat(inputFeedback.value));

            function loop() {
                try {
                    renderer.update(); // 物理更新
                    renderer.render(); // 描画
                    requestAnimationFrame(loop);
                } catch (err) {
                    console.error("Render loop error:", err);
                }
            }
            requestAnimationFrame(loop);
            
        } catch (e) {
            console.error(e);
            alert("WebGPU initialization failed. Please ensure your browser supports WebGPU.\n\nError: " + e);
            btn.disabled = false;
            btn.innerText = "Initialization Failed";
        }
    });
}

run();