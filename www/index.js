import init, { QuantumRenderer } from './pkg/inverse_observation_induced_probability_field_interference.js';

async function run() {
    await init();
    
    const btn = document.getElementById('start-btn');
    const canvas = document.getElementById('quantum-canvas');
    
    btn.addEventListener('click', async () => {
        btn.disabled = true;
        btn.innerText = "Quantum Coherence Established";
        
        try {
            // Rust側のWGPU初期化
            const renderer = await QuantumRenderer.new("quantum-canvas");
            
            function loop() {
                renderer.update(); // 物理更新
                renderer.render(); // 描画
                requestAnimationFrame(loop);
            }
            requestAnimationFrame(loop);
            
        } catch (e) {
            console.error(e);
            alert("WebGPU not supported or error initializing: " + e);
            btn.disabled = false;
        }
    });
}

run();