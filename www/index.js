import init, { compute_metrics_js } from './pkg/nx_compute_rs.js';

async function run() {
    await init(); // Initialize WASM
    
    const btn = document.getElementById('run-btn');
    const output = document.getElementById('output');
    
    btn.innerText = "Run Core Algorithm (10M iters)";
    btn.disabled = false;

    btn.addEventListener('click', () => {
        output.innerText = "Computing...";
        
        // Use setTimeout to allow UI to update before blocking main thread
        setTimeout(() => {
            const start = performance.now();
            
            // Call Rust function
            const result = compute_metrics_js(10_000_000n, 1.5);
            
            const end = performance.now();
            output.innerText = `Result: ${result.toFixed(6)}\nTime: ${(end - start).toFixed(2)} ms`;
        }, 10);
    });
}

run();
