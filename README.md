# Rotor

A real-time 3D UAV tracking interface built with Svelte and Three.js. Displays live position data for multiple drones across X, Y, and Z coordinates in a futuristic heads-up display.

## Stack

- **`ui/`** — Svelte + Vite + Three.js
- **`server/`** — Rust (sensor pipeline, gateway, message broker)

## Development

**UI**
```bash
cd ui
npm install
npm run dev
```

**Server**
```bash
cd server
cargo run --bin gateway
```
