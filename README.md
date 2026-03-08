# 🔥 Oxide

**A Rust frontend framework that compiles to WebAssembly.**

Write reactive browser apps entirely in Rust — no JavaScript. Fine-grained signals, direct DOM updates, and a modern `view!` macro with conditionals, loops, and two-way binding.

**[Live Demo](https://ievangelist.github.io/Oxide/) · [Playground](https://ievangelist.github.io/Oxide/playground.html) · [Docs](https://ievangelist.github.io/Oxide/docs.html)**

```rust
use oxide::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    mount("#app", || {
        let mut count = signal(0);

        view! {
            <div>
                <p>"Count: " {count}</p>
                <button on:click={move |_: oxide::dom::Event| count += 1}>
                    "Increment"
                </button>
            </div>
        }
    });
}
```

## Quick Start

```sh
# Install toolchain
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Install the Oxide CLI
cargo install --git https://github.com/IEvangelist/Oxide oxide-cli

# Create and run a project
oxide new my-app
cd my-app
oxide dev        # → http://localhost:8080 with live reload
```

## CLI

| Command | Description |
|---|---|
| `oxide new <name>` | Scaffold a new project with template, deps, and index.html |
| `oxide dev` | Dev server with live reload + WASM debug info (DWARF) |
| `oxide build` | Production build via wasm-pack, optimized with wasm-opt |
| `oxide serve` | Serve the built site locally |

**Debugging:** `oxide dev` builds with DWARF debug info. Install Chrome's ["C/C++ DevTools Support (DWARF)"](https://chrome.google.com/webstore/detail/cc%2B%2B-devtools-support-dwa/pdcpmagijalfljmkmjngeonclgbbannb) extension to set breakpoints and step through Rust source in the browser.

## Architecture

```
oxide/
├── crates/
│   ├── oxide-core         # Signals, effects, batching, context
│   ├── oxide-macros        # view! proc macro (if/for/bind/class)
│   ├── oxide-dom           # DOM renderer + utilities (web-sys)
│   ├── oxide-telemetry     # OpenTelemetry tracing
│   ├── oxide-resiliency    # Error boundaries, retry, circuit breaker
│   ├── oxide-cli           # CLI toolchain (new/dev/build/serve)
│   └── oxide               # Facade — re-exports everything
└── examples/
    ├── counter             # Minimal counter demo (30 KB)
    └── showcase            # 18-demo marketing site (219 KB)
```

## `view!` Macro

```rust
view! {
    <div class="card" class:active={is_active.get()}>
        // Static text
        <h1>"Hello"</h1>

        // Reactive expression
        <p>{count}</p>

        // Dynamic attributes
        <div class={dynamic_class}>"styled"</div>

        // Two-way binding
        <input bind:value={name} />
        <input type="checkbox" bind:checked={subscribed} />

        // Conditional rendering
        {if show.get() {
            <p>"Visible!"</p>
        } else {
            <p>"Hidden"</p>
        }}

        // List rendering
        {for item in items.get() {
            <li>{item}</li>
        }}

        // Events
        <button on:click={move |_: oxide::dom::Event| count += 1}>
            "Click"
        </button>

        // Components
        <MyComponent prop={value} />
    </div>
}
```

## Signals & Reactivity

```rust
let count = signal(0);           // Create a signal
let val = count.get();            // Read (auto-tracks in effects)
count.set(5);                     // Write (notifies subscribers)
count.update(|n| *n += 1);       // Mutate in-place

let doubled = memo(move || count.get() * 2);  // Derived signal

create_effect(move || {           // Auto-tracking effect
    log(&format!("Count: {}", count.get()));
});

batch(|| {                        // Coalesce updates
    a.set(1);
    b.set(2);
    // Effects run once, not twice
});
```

`Signal<T>` is `Copy` — just an ID into a thread-local arena. No `Rc`, no `clone()`.

## OpenTelemetry

Built-in observability with zero overhead when disabled:

```rust
use oxide::telemetry;

telemetry::init(telemetry::Config {
    service_name: "my-app",
    endpoint: None, // console mode
    ..Default::default()
});

// All signal reads/writes and effect runs are now traced automatically.
// Use traced_fetch() for W3C trace context propagation:
let data = telemetry::traced_fetch("https://api.example.com/data").await?;
```

## Resiliency

Production-grade fault tolerance patterns:

```rust
use oxide::resiliency::*;

// Catch panics, render fallback UI
let el = error_boundary(
    || risky_component(),
    |err| view! { <p>"Error: " {err}</p> },
);

// Retry with exponential backoff
let data = retry(RetryConfig::exponential(3, 1000), || {
    fetch_data()
}).await?;

// Circuit breaker
let breaker = CircuitBreaker::new(CircuitBreakerConfig {
    failure_threshold: 5,
    reset_timeout_ms: 30_000,
});
let result = breaker.call(|| fetch_data()).await;
```

## License

MIT
