/* tslint:disable */
/* eslint-disable */

export class QuantumRenderer {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  set_wave_number(val: number): void;
  set_feedback_strength(val: number): void;
  static new(canvas_id: string): Promise<QuantumRenderer>;
  render(): void;
  update(): void;
  wave_number: number;
  feedback_strength: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_get_quantumrenderer_feedback_strength: (a: number) => number;
  readonly __wbg_get_quantumrenderer_wave_number: (a: number) => number;
  readonly __wbg_quantumrenderer_free: (a: number, b: number) => void;
  readonly __wbg_set_quantumrenderer_feedback_strength: (a: number, b: number) => void;
  readonly __wbg_set_quantumrenderer_wave_number: (a: number, b: number) => void;
  readonly quantumrenderer_new: (a: number, b: number) => any;
  readonly quantumrenderer_render: (a: number) => void;
  readonly quantumrenderer_set_feedback_strength: (a: number, b: number) => void;
  readonly quantumrenderer_set_wave_number: (a: number, b: number) => void;
  readonly quantumrenderer_update: (a: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h5ba262fcf01eed2e: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h6e2f67d062ddca88: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__he4fb1ff93077e3b6: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hc18561b8cd25897a: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hc2d3a4d986077f65: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
