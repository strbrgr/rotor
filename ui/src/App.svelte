<script lang="ts">
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

  type UavState = {
    hex: string;
    label: string;
    x: number;
    y: number;
    z: number;
  };

  type Drone3D = {
    mesh: THREE.Mesh;
    trailGeo: THREE.BufferGeometry;
    buf: Float32Array;
    count: number;
  };

  type UavReading = {
    id: string;
    x: number;
    y: number;
    z: number;
    created_at: number;
  };

  const PALETTE = [
    { hex: '#1a1a1a', color: 0x1a1a1a },
    { hex: '#555555', color: 0x555555 },
    { hex: '#888888', color: 0x888888 },
  ];

  const TRAIL_LEN = 90;
  const SSE_URL = 'http://127.0.0.1:3001/events';

  let canvas: HTMLCanvasElement;
  let uavs = $state<Record<string, UavState>>({});

  const fmt = (n: number): string => (n >= 0 ? ' ' : '') + n.toFixed(2);

  onMount(() => {
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(window.innerWidth, window.innerHeight);

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xf0f0f0);
    scene.fog = new THREE.FogExp2(0xf0f0f0, 0.007);

    const camera = new THREE.PerspectiveCamera(55, window.innerWidth / window.innerHeight, 0.1, 800);
    camera.position.set(90, 65, 90);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.set(0, 22, 0);
    controls.enableDamping = true;
    controls.dampingFactor = 0.06;
    controls.minDistance = 20;
    controls.maxDistance = 350;
    controls.update();

    scene.add(new THREE.AmbientLight(0xffffff, 5));

    const mkGrid = (size: number, divs: number, cx: number, c: number): THREE.GridHelper =>
      new THREE.GridHelper(size, divs, cx, c);

    scene.add(mkGrid(240, 24, 0x777777, 0xbbbbbb));
    const gridXY = mkGrid(240, 24, 0x888888, 0xcccccc);
    gridXY.rotation.x = Math.PI / 2;
    scene.add(gridXY);
    const gridYZ = mkGrid(240, 24, 0x888888, 0xcccccc);
    gridYZ.rotation.z = Math.PI / 2;
    scene.add(gridYZ);

    const mkLine = (to: THREE.Vector3, color: number): THREE.Line => {
      const g = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), to]);
      return new THREE.Line(g, new THREE.LineBasicMaterial({ color, fog: false }));
    };
    scene.add(mkLine(new THREE.Vector3(70, 0, 0), 0xcc2222));
    scene.add(mkLine(new THREE.Vector3(0, 70, 0), 0x228822));
    scene.add(mkLine(new THREE.Vector3(0, 0, 70), 0x2244cc));

    const boxGeo = new THREE.BoxGeometry(2.5, 2.5, 2.5);
    const edgeGeo = new THREE.EdgesGeometry(boxGeo);

    const scene3d = new Map<string, Drone3D>();
    let paletteIndex = 0;

    const registerUav = (id: string, color: number, hex: string): void => {
      const mesh = new THREE.Mesh(
        boxGeo,
        new THREE.MeshStandardMaterial({
          color,
          emissive: color,
          emissiveIntensity: 0.55,
          metalness: 0.4,
          roughness: 0.3,
        })
      );
      mesh.add(new THREE.LineSegments(edgeGeo, new THREE.LineBasicMaterial({ color })));
      mesh.add(new THREE.PointLight(color, 3, 25));
      scene.add(mesh);

      const buf = new Float32Array(TRAIL_LEN * 3);
      const trailGeo = new THREE.BufferGeometry();
      trailGeo.setAttribute('position', new THREE.BufferAttribute(buf, 3));
      trailGeo.setDrawRange(0, 0);
      scene.add(new THREE.Line(
        trailGeo,
        new THREE.LineBasicMaterial({ color, transparent: true, opacity: 0.38 }),
      ));

      scene3d.set(id, { mesh, trailGeo, buf, count: 0 });
      uavs[id] = { hex, label: id.slice(0, 8).toUpperCase(), x: 0, y: 0, z: 0 };
    };

    const applyReading = (id: string, x: number, y: number, z: number): void => {
      const d = scene3d.get(id)!;
      d.mesh.position.set(x, y, z);

      const b = d.buf;
      b.copyWithin(3, 0, (TRAIL_LEN - 1) * 3);
      b[0] = x; b[1] = y; b[2] = z;
      d.trailGeo.attributes.position.needsUpdate = true;
      d.count = Math.min(d.count + 1, TRAIL_LEN);
      d.trailGeo.setDrawRange(0, d.count);

      uavs[id].x = x;
      uavs[id].y = y;
      uavs[id].z = z;
    };

    const source = new EventSource(SSE_URL);

    source.onmessage = (e: MessageEvent): void => {
      const reading = JSON.parse(e.data) as UavReading;

      if (!scene3d.has(reading.id)) {
        const { hex, color } = PALETTE[paletteIndex % PALETTE.length];
        paletteIndex++;
        registerUav(reading.id, color, hex);
      }

      applyReading(reading.id, reading.x, reading.y, reading.z);
    };

    source.onerror = (): void => {
      console.warn('SSE connection lost, browser will retry automatically');
    };

    const onResize = (): void => {
      camera.aspect = window.innerWidth / window.innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(window.innerWidth, window.innerHeight);
    };
    window.addEventListener('resize', onResize);

    const clock = new THREE.Clock();
    let rafId: number;

    const tick = (): void => {
      rafId = requestAnimationFrame(tick);
      const t = clock.getElapsedTime();
      scene3d.forEach((d) => {
        d.mesh.rotation.y = t * 0.9;
        d.mesh.rotation.x = t * 0.45;
      });
      controls.update();
      renderer.render(scene, camera);
    };
    tick();

    return () => {
      source.close();
      cancelAnimationFrame(rafId);
      window.removeEventListener('resize', onResize);
      renderer.dispose();
    };
  });
</script>

<canvas bind:this={canvas}></canvas>

<div class="ui">
  <div class="scanlines"></div>

  <header class="topbar">
    <div class="brand">
      <span class="dim">[</span>&nbsp;UAV&nbsp;TRACKER&nbsp;<span class="dim">]</span>
    </div>
    <div class="sys-status">
      <span class="pulse-dot"></span>
      {#if Object.keys(uavs).length === 0}
        AWAITING&nbsp;SENSORS
      {:else}
        {Object.keys(uavs).length}&nbsp;UNIT{Object.keys(uavs).length !== 1 ? 'S' : ''}&nbsp;ACTIVE
      {/if}
    </div>
    <div class="axis-legend">
      <span class="ax-x">■ X</span>
      <span class="ax-y">■ Y</span>
      <span class="ax-z">■ Z</span>
    </div>
  </header>

  <aside class="panel">
    <div class="panel-label">// ACTIVE UNITS</div>
    {#each Object.entries(uavs) as [, uav]}
      <div class="card" style="--c: {uav.hex}">
        <div class="card-head">
          <span class="indicator"></span>
          <span class="uav-id">UAV-{uav.label}</span>
          <span class="active-badge">● ACTIVE</span>
        </div>
        <div class="coords">
          <div class="row"><span class="lbl ax-x">X</span><span class="val">{fmt(uav.x)}</span><span class="unit">m</span></div>
          <div class="row"><span class="lbl ax-y">Y</span><span class="val">{fmt(uav.y)}</span><span class="unit">m</span></div>
          <div class="row"><span class="lbl ax-z">Z</span><span class="val">{fmt(uav.z)}</span><span class="unit">m</span></div>
        </div>
      </div>
    {/each}
  </aside>

  <div class="corner tl"></div>
  <div class="corner tr"></div>
  <div class="corner bl"></div>
  <div class="corner br"></div>

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
    color: #1a1a1a;
    z-index: 10;
  }

  .scanlines {
    display: none;
  }

  .topbar {
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    background: rgba(240, 240, 240, 0.88);
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(6px);
  }

  .brand {
    font-size: 14px;
    color: #1a1a1a;
    letter-spacing: 0.22em;
  }
  .dim { opacity: 0.3; }

  .sys-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10px;
    opacity: 0.5;
  }

  .pulse-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #333333;
    animation: blink 1.6s ease-in-out infinite;
    flex-shrink: 0;
  }

  .axis-legend {
    display: flex;
    gap: 14px;
    font-size: 10px;
    opacity: 0.55;
  }
  .ax-x { color: #cc2222; }
  .ax-y { color: #228822; }
  .ax-z { color: #2244cc; }

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
    color: rgba(0, 0, 0, 0.35);
    letter-spacing: 0.18em;
    padding-bottom: 6px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .card {
    background: rgba(245, 245, 245, 0.88);
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-left: 2px solid var(--c, #333333);
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
    background: var(--c, #333333);
    flex-shrink: 0;
  }

  .uav-id {
    flex: 1;
    font-size: 11px;
    color: #1a1a1a;
    letter-spacing: 0.12em;
  }

  .active-badge {
    font-size: 9px;
    color: #555555;
    letter-spacing: 0.1em;
    animation: blink 2.2s ease-in-out infinite;
  }

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
    color: #1a1a1a;
    font-variant-numeric: tabular-nums;
    white-space: pre;
  }

  .unit {
    width: 12px;
    font-size: 9px;
    opacity: 0.35;
  }

  .corner {
    position: absolute;
    width: 22px;
    height: 22px;
  }
  .corner::before,
  .corner::after {
    content: '';
    position: absolute;
    background: rgba(0, 0, 0, 0.3);
  }
  .corner::before { width: 100%; height: 1px; top: 0; left: 0; }
  .corner::after  { width: 1px; height: 100%; top: 0; left: 0; }

  .tl { top: 8px;    left: 8px; }
  .tr { top: 8px;    right: 8px;    transform: scaleX(-1); }
  .bl { bottom: 8px; left: 8px;     transform: scaleY(-1); }
  .br { bottom: 8px; right: 8px;    transform: scale(-1); }

  .hint {
    position: absolute;
    bottom: 14px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 9px;
    letter-spacing: 0.14em;
    opacity: 0.3;
    white-space: nowrap;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.15; }
  }
</style>
