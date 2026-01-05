/* tslint:disable */
/* eslint-disable */

export class Playground {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get available choices as JSON.
   */
  get_choices(): any;
  /**
   * Get all variables as JSON.
   */
  get_variables(): any;
  /**
   * Select a choice by index.
   */
  select_choice(index: number): void;
  /**
   * Get the current passage name.
   */
  get_passage_name(): string | undefined;
  /**
   * Create a new playground instance.
   */
  constructor();
  /**
   * Parse a scene from source code.
   */
  parse(source: string): void;
  /**
   * Start or restart the scene from the beginning.
   */
  start(): void;
  /**
   * Add a "has" item (e.g., inventory item).
   */
  add_has(category: string, key: string): void;
  /**
   * Set a variable value before starting.
   */
  set_var(name: string, value: string): void;
  /**
   * Check if the scene has ended.
   */
  is_ended(): boolean;
  /**
   * Get effect calls made during this run.
   */
  get_calls(): any;
  /**
   * Get the current prose text.
   */
  get_prose(): string | undefined;
  /**
   * Get the scene title.
   */
  get_title(): string | undefined;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_playground_free: (a: number, b: number) => void;
  readonly playground_add_has: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly playground_get_calls: (a: number) => any;
  readonly playground_get_choices: (a: number) => any;
  readonly playground_get_passage_name: (a: number) => [number, number];
  readonly playground_get_prose: (a: number) => [number, number];
  readonly playground_get_title: (a: number) => [number, number];
  readonly playground_get_variables: (a: number) => any;
  readonly playground_is_ended: (a: number) => number;
  readonly playground_new: () => number;
  readonly playground_parse: (a: number, b: number, c: number) => [number, number];
  readonly playground_select_choice: (a: number, b: number) => [number, number];
  readonly playground_set_var: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly playground_start: (a: number) => [number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
