# 6D Quantum Gaussian Splatting

[![CI/CD](https://github.com/Funmatu/Inverse-Observation-Induced-Probability-Field-Interference/actions/workflows/deploy.yml/badge.svg)](https://funmatu.github.io/Inverse-Observation-Induced-Probability-Field-Interference/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Python](https://img.shields.io/badge/python-3670A0?style=for-the-badge&logo=python&logoColor=ffdd54)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white)

![WGPU](https://img.shields.io/badge/GPU-WGPU%20Compute-blueviolet?style=flat-square&logo=webgpu&logoColor=white)
![PyO3](https://img.shields.io/badge/Bindings-PyO3-blue?style=flat-square)
![Maturin](https://img.shields.io/badge/Build-Maturin-green?style=flat-square)

![Platform](https://img.shields.io/badge/platform-Web%20%7C%20Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)

**A Proof of Concept for Inverse Observation-Induced Probability Field Interference.**

This project explores a novel SLAM architecture inspired by Quantum Mechanics and the concept of "Temporal Pincer Movements". Instead of filtering current observations to estimate position, we model the environment as a field of observers that project "probability waves" back into the space. The convergence of these waves (Constructive Interference) defines the agent's existence.

## 1. Theoretical Foundation

### 1.1 Inverse Observation Model
Traditional SLAM solves $P(x | z)$ (Probability of pose $x$ given measurement $z$).
We invert this: The environment consists of anchors $L_i$ that "observe" the agent. Each anchor emits a spherical probability wave $\Psi_i(x)$ based on the measured distance $d_i$.

$$
\Psi(x) = \sum_{i} A_i \cdot e^{i (k |x - L_i| - \phi_i)}
$$

* **Constructive Interference:** Where waves align, probability density $|\Psi|^2$ spikes. This is the estimated position.
* **Destructive Interference:** Elsewhere, waves cancel out, naturally suppressing noise and "ghost" solutions.

### 1.2 Feedback (Temporal Entanglement)
We introduce a 6th dimension (Time/Causality) by feeding the *past* probability field back into the *current* estimation.

$$
P_{t}(x) = (1 - \alpha)|\Psi_{t}(x)|^2 + \alpha P_{t-1}(x)
$$

This creates a "World Tube" where the agent's existence is stabilized by its own history, preventing instant tracking loss (teleportation).

## 2. Architecture

This repository uses a **Dual-Runtime Architecture** powered by Rust.

| Component | Tech Stack | Role |
|-----------|------------|------|
| **Core Physics** | Rust (CPU) | Exact math verification, unit testing. |
| **Visualization** | Rust + WGPU (Compute Shader) | Real-time interference simulation, massive parallelization. |
| **Analysis** | Python (PyO3) | Automated testing of interference properties. |
| **Web Demo** | WASM + WebGPU | Browser-based interactive visualization. |

## 3. Implementation Details

### Compute Shader (`shader.wgsl`)
The heart of the simulation. It runs on the GPU, calculating complex wave summation for every pixel in parallel.
* **Ping-Pong Buffering:** Used to read the previous frame's probability texture while writing to the current one, enabling the temporal feedback loop.
* **Complex Math:** Standard WGSL `float` operations are combined to simulate complex number arithmetic (Phase/Amplitude).

### Hybrid Rust Crate (`lib.rs`)
* **`QuantumSlamCore`:** A pure CPU implementation of the interference formula. Exposed to Python for `pytest`.
* **`QuantumRenderer`:** A WGPU wrapper handling the device, queue, and swapchain for WebAssembly.

## 4. Running the Demo

### Web (Visualization)
Requires a browser with **WebGPU** support (Chrome/Edge 113+).

1.  `wasm-pack build --target web --features wasm`
2.  Serve the `www` directory.

### Python (Verification)
1.  `maturin develop --features python`
2.  `pytest test_core.py`

## 5. Future Work: The "Temporal Pincer Movements" Algorithm
Currently, the feedback is $t-1 \to t$. The next step is to implement **Bi-directional Time Optimization**:
Using loop closures (future information) to propagate probability waves *backwards* in time ($t+k \to t$), collapsing the wave function of past uncertain states.

---