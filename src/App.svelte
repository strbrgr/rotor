<script>
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

  let canvas;

  // Reactive HUD data — updated each animation frame
  let positions = $state([
    { x: 0, y: 0, z: 0 },
    { x: 0, y: 0, z: 0 },
    { x: 0, y: 0, z: 0 },
  ]);

  const DRONES = [
    {
      id: 'ALPHA-01', hex: '#00f0ff', color: 0x00f0ff,
      ax: 42, ay: 14, az: 36, fx: 0.37, fy: 0.61, fz: 0.29,
      px: 0.0, py: 0.0, pz: 1.0, baseY: 22,
    },
    {
      id: 'BETA-02', hex: '#ff3cac', color: 0xff3cac,
      ax: 32, ay: 18, az: 44, fx: 0.53, fy: 0.44, fz: 0.47,
      px: 2.1, py: 1.7, pz: 0.4, baseY: 28,
    },
    {
      id: 'GAMMA-03', hex: '#39ff14', color: 0x39ff14,
      ax: 28, ay: 22, az: 33, fx: 0.68, fy: 0.52, fz: 0.73,
      px: 1.2, py: 3.1, pz: 2.3, baseY: 16,
    },
  ];

  const TRAIL_LEN = 90;

  onMount(() => {
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(window.innerWidth, window.innerHeight);

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x050a0f);
    scene.fog = new THREE.FogExp2(0x050a0f, 0.007);

    const camera = new THREE.PerspectiveCamera(55, window.innerWidth / window.innerHeight, 0.1, 800);
    camera.position.set(90, 65, 90);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.set(0, 22, 0);
    controls.enableDamping = true;
    controls.dampingFactor = 0.06;
    controls.minDistance = 20;
    controls.maxDistance = 350;
    controls.update();

    scene.add(new THREE.AmbientLight(0x0a1a2a, 4));

    // Reference grids — three axis-aligned planes at origin
    const mkGrid = (size, divs, cx, c) => new THREE.GridHelper(size, divs, cx, c);

    // XZ — floor (horizontal)
    scene.add(mkGrid(240, 48, 0x888888, 0x444444));

    // XY — back wall (rotate 90° around X)
    const gridXY = mkGrid(240, 48, 0x666666, 0x333333);
    gridXY.rotation.x = Math.PI / 2;
    scene.add(gridXY);

    // YZ — side wall (rotate 90° around Z)
    const gridYZ = mkGrid(240, 48, 0x666666, 0x333333);
    gridYZ.rotation.z = Math.PI / 2;
    scene.add(gridYZ);

    // Axis lines from origin — brighter so they read over the grids
    const mkLine = (to, color) => {
      const g = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), to]);
      return new THREE.Line(g, new THREE.LineBasicMaterial({ color }));
    };
    scene.add(mkLine(new THREE.Vector3(70, 0, 0), 0xff3344));  // X — red
    scene.add(mkLine(new THREE.Vector3(0, 70, 0), 0x33ff77));  // Y — green
    scene.add(mkLine(new THREE.Vector3(0, 0, 70), 0x3377ff));  // Z — blue

    // Shared geometry
    const boxGeo = new THREE.BoxGeometry(2.5, 2.5, 2.5);
    const edgeGeo = new THREE.EdgesGeometry(boxGeo);

    const drones3d = DRONES.map((d) => {
      const mesh = new THREE.Mesh(
        boxGeo,
        new THREE.MeshStandardMaterial({
          color: d.color,
          emissive: d.color,
          emissiveIntensity: 0.55,
          metalness: 0.4,
          roughness: 0.3,
        })
      );
      // Crisp edge wireframe on top of the cube
      mesh.add(new THREE.LineSegments(edgeGeo, new THREE.LineBasicMaterial({ color: d.color })));
      // Local glow
      mesh.add(new THREE.PointLight(d.color, 3, 25));
      scene.add(mesh);

      // Trail line
      const buf = new Float32Array(TRAIL_LEN * 3);
      const trailGeo = new THREE.BufferGeometry();
      trailGeo.setAttribute('position', new THREE.BufferAttribute(buf, 3));
      trailGeo.setDrawRange(0, 0);
      scene.add(new THREE.Line(
        trailGeo,
        new THREE.LineBasicMaterial({ color: d.color, transparent: true, opacity: 0.38 }),
      ));

      return { mesh, trailGeo, buf, count: 0 };
    });

    const onResize = () => {
      camera.aspect = window.innerWidth / window.innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(window.innerWidth, window.innerHeight);
    };
    window.addEventListener('resize', onResize);

    const clock = new THREE.Clock();
    let rafId;

    const tick = () => {
      rafId = requestAnimationFrame(tick);
      const t = clock.getElapsedTime();

      DRONES.forEach((d, i) => {
        const x = d.ax * Math.sin(t * d.fx + d.px);
        const y = d.baseY + d.ay * Math.cos(t * d.fy + d.py);
        const z = d.az * Math.sin(t * d.fz + d.pz);

        drones3d[i].mesh.position.set(x, y, z);
        drones3d[i].mesh.rotation.y = t * 0.9 + i * 2.1;
        drones3d[i].mesh.rotation.x = t * 0.45 + i;

        // Prepend new position to trail buffer
        const b = drones3d[i].buf;
        b.copyWithin(3, 0, (TRAIL_LEN - 1) * 3);
        b[0] = x; b[1] = y; b[2] = z;
        drones3d[i].trailGeo.attributes.position.needsUpdate = true;
        drones3d[i].count = Math.min(drones3d[i].count + 1, TRAIL_LEN);
        drones3d[i].trailGeo.setDrawRange(0, drones3d[i].count);

        // Push to HUD
        positions[i].x = x;
        positions[i].y = y;
        positions[i].z = z;
      });

      controls.update();
      renderer.render(scene, camera);
    };
    tick();

    return () => {
      cancelAnimationFrame(rafId);
      window.removeEventListener('resize', onResize);
      renderer.dispose();
    };
  });

  const fmt = (n) => (n >= 0 ? ' ' : '') + n.toFixed(2);
</script>

<canvas bind:this={canvas}></canvas>

<div class="ui">
  <!-- Scanline overlay -->
  <div class="scanlines"></div>

  <!-- Top bar -->
  <header class="topbar">
    <div class="brand">
      <span class="dim">[</span>&nbsp;DRONE&nbsp;TRACKER&nbsp;<span class="dim">]</span>
    </div>
    <div class="sys-status">
      <span class="pulse-dot"></span>
      SYSTEM&nbsp;ONLINE&nbsp;//&nbsp;3&nbsp;UNITS&nbsp;ACTIVE
    </div>
    <div class="axis-legend">
      <span class="ax-x">■ X</span>
      <span class="ax-y">■ Y</span>
      <span class="ax-z">■ Z</span>
    </div>
  </header>

  <!-- Drone panel -->
  <aside class="panel">
    <div class="panel-label">// ACTIVE UNITS</div>
    {#each DRONES as d, i}
      <div class="card" style="--c: {d.hex}">
        <div class="card-head">
          <span class="indicator"></span>
          <span class="uav-id">UAV-{d.id}</span>
          <span class="active-badge">● ACTIVE</span>
        </div>
        <div class="coords">
          <div class="row"><span class="lbl ax-x">X</span><span class="val">{fmt(positions[i].x)}</span><span class="unit">m</span></div>
          <div class="row"><span class="lbl ax-y">Y</span><span class="val">{fmt(positions[i].y)}</span><span class="unit">m</span></div>
          <div class="row"><span class="lbl ax-z">Z</span><span class="val">{fmt(positions[i].z)}</span><span class="unit">m</span></div>
        </div>
      </div>
    {/each}
  </aside>

  <!-- Corner frame decorations -->
  <div class="corner tl"></div>
  <div class="corner tr"></div>
  <div class="corner bl"></div>
  <div class="corner br"></div>

  <!-- Bottom hint -->
  <div class="hint">DRAG TO ORBIT&nbsp;·&nbsp;SCROLL TO ZOOM&nbsp;·&nbsp;RIGHT-DRAG TO PAN</div>
</div>

<style>
  canvas {
    display: block;
    width: 100vw;
    height: 100vh;
  }

  .ui {
    position: fixed;
    inset: 0;
    pointer-events: none;
    font-family: 'Share Tech Mono', 'Courier New', monospace;
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: rgba(200, 230, 240, 0.8);
    z-index: 10;
  }

  /* Subtle scanline texture */
  .scanlines {
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(
      0deg,
      transparent,
      transparent 3px,
      rgba(0, 0, 0, 0.04) 3px,
      rgba(0, 0, 0, 0.04) 4px
    );
    pointer-events: none;
  }

  /* ─── Top bar ─── */
  .topbar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    background: rgba(5, 12, 20, 0.72);
    border-bottom: 1px solid rgba(0, 240, 255, 0.14);
    backdrop-filter: blur(6px);
  }

  .brand {
    font-size: 14px;
    color: #00f0ff;
    letter-spacing: 0.22em;
  }
  .dim { opacity: 0.35; }

  .sys-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10px;
    opacity: 0.55;
  }

  .pulse-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #44ff88;
    animation: blink 1.6s ease-in-out infinite;
    flex-shrink: 0;
  }

  .axis-legend {
    display: flex;
    gap: 14px;
    font-size: 10px;
    opacity: 0.6;
  }
  .ax-x { color: #ff4455; }
  .ax-y { color: #44ff88; }
  .ax-z { color: #4488ff; }

  /* ─── Drone panel ─── */
  .panel {
    position: absolute;
    top: 62px;
    left: 16px;
    width: 224px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .panel-label {
    font-size: 9px;
    color: rgba(0, 240, 255, 0.35);
    letter-spacing: 0.18em;
    padding-bottom: 6px;
    border-bottom: 1px solid rgba(0, 240, 255, 0.08);
  }

  .card {
    background: rgba(5, 14, 24, 0.78);
    border: 1px solid rgba(0, 240, 255, 0.12);
    border-left: 2px solid var(--c, #00f0ff);
    padding: 10px 12px;
    backdrop-filter: blur(5px);
    clip-path: polygon(0 0, calc(100% - 10px) 0, 100% 10px, 100% 100%, 0 100%);
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-bottom: 9px;
  }

  .indicator {
    width: 7px;
    height: 7px;
    border-radius: 1px;
    background: var(--c, #00f0ff);
    box-shadow: 0 0 7px var(--c, #00f0ff);
    flex-shrink: 0;
  }

  .uav-id {
    flex: 1;
    font-size: 11px;
    color: rgba(200, 230, 240, 0.9);
    letter-spacing: 0.12em;
  }

  .active-badge {
    font-size: 9px;
    color: #44ff88;
    letter-spacing: 0.1em;
    animation: blink 2.2s ease-in-out infinite;
  }

  /* Coordinate rows */
  .coords { display: flex; flex-direction: column; gap: 4px; }

  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
  }

  .lbl {
    width: 12px;
    text-align: center;
    font-size: 10px;
    font-weight: bold;
    flex-shrink: 0;
  }

  .val {
    flex: 1;
    text-align: right;
    color: rgba(220, 240, 250, 0.92);
    font-variant-numeric: tabular-nums;
    white-space: pre;
  }

  .unit {
    width: 12px;
    font-size: 9px;
    opacity: 0.35;
  }

  /* ─── Corner decorations ─── */
  .corner {
    position: absolute;
    width: 22px;
    height: 22px;
  }
  .corner::before,
  .corner::after {
    content: '';
    position: absolute;
    background: rgba(0, 240, 255, 0.45);
  }
  .corner::before { width: 100%; height: 1px; top: 0; left: 0; }
  .corner::after  { width: 1px; height: 100%; top: 0; left: 0; }

  .tl { top: 8px;    left: 8px; }
  .tr { top: 8px;    right: 8px;    transform: scaleX(-1); }
  .bl { bottom: 8px; left: 8px;     transform: scaleY(-1); }
  .br { bottom: 8px; right: 8px;    transform: scale(-1); }

  /* ─── Bottom hint ─── */
  .hint {
    position: absolute;
    bottom: 14px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 9px;
    letter-spacing: 0.14em;
    opacity: 0.25;
    white-space: nowrap;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.15; }
  }
</style>
