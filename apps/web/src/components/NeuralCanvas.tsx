"use client";
import { useEffect, useRef } from "react";
import * as THREE from "three";

export default function NeuralCanvas() {
  const mountRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = mountRef.current;
    if (!el) return;

    // ── Scene setup ──────────────────────────────────────────────
    const W = el.clientWidth;
    const H = el.clientHeight;
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(60, W / H, 0.1, 2000);
    camera.position.set(0, 0, 120);

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(W, H);
    renderer.setClearColor(0x000000, 0);
    el.appendChild(renderer.domElement);

    // ── Particles ────────────────────────────────────────────────
    const COUNT = 1400;
    const positions = new Float32Array(COUNT * 3);
    const colors = new Float32Array(COUNT * 3);
    const sizes = new Float32Array(COUNT);
    const velocities: THREE.Vector3[] = [];

    const palette = [
      new THREE.Color("#6366f1"),
      new THREE.Color("#818cf8"),
      new THREE.Color("#a78bfa"),
      new THREE.Color("#8b5cf6"),
      new THREE.Color("#4f46e5"),
    ];

    for (let i = 0; i < COUNT; i++) {
      const r = 80 + Math.random() * 60;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      positions[i * 3]     = r * Math.sin(phi) * Math.cos(theta);
      positions[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta) * 0.6;
      positions[i * 3 + 2] = r * Math.cos(phi);

      const c = palette[Math.floor(Math.random() * palette.length)];
      colors[i * 3]     = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;

      sizes[i] = Math.random() * 2.5 + 0.5;
      velocities.push(new THREE.Vector3(
        (Math.random() - 0.5) * 0.02,
        (Math.random() - 0.5) * 0.01,
        (Math.random() - 0.5) * 0.02,
      ));
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geo.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    geo.setAttribute("size", new THREE.BufferAttribute(sizes, 1));

    const mat = new THREE.ShaderMaterial({
      vertexShader: `
        attribute float size;
        attribute vec3 color;
        varying vec3 vColor;
        varying float vAlpha;
        void main() {
          vColor = color;
          vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
          vAlpha = 1.0 - smoothstep(80.0, 150.0, -mvPos.z);
          gl_PointSize = size * (250.0 / -mvPos.z);
          gl_Position = projectionMatrix * mvPos;
        }
      `,
      fragmentShader: `
        varying vec3 vColor;
        varying float vAlpha;
        void main() {
          float d = length(gl_PointCoord - vec2(0.5));
          if (d > 0.5) discard;
          float alpha = (1.0 - smoothstep(0.0, 0.5, d)) * vAlpha * 0.85;
          gl_FragColor = vec4(vColor, alpha);
        }
      `,
      transparent: true,
      vertexColors: true,
      depthWrite: false,
    });

    const points = new THREE.Points(geo, mat);
    scene.add(points);

    // ── Neural connections ────────────────────────────────────────
    const lineGeo = new THREE.BufferGeometry();
    const lineVerts: number[] = [];
    const lineColors: number[] = [];
    const threshold = 28;

    for (let i = 0; i < COUNT; i++) {
      for (let j = i + 1; j < COUNT; j++) {
        const dx = positions[i*3]   - positions[j*3];
        const dy = positions[i*3+1] - positions[j*3+1];
        const dz = positions[i*3+2] - positions[j*3+2];
        const dist = Math.sqrt(dx*dx + dy*dy + dz*dz);
        if (dist < threshold) {
          const alpha = 1 - dist / threshold;
          lineVerts.push(
            positions[i*3], positions[i*3+1], positions[i*3+2],
            positions[j*3], positions[j*3+1], positions[j*3+2],
          );
          const r = 0.39 + alpha * 0.2;
          const g = 0.40 + alpha * 0.1;
          const b = 0.95;
          lineColors.push(r, g, b, r, g, b);
        }
      }
    }

    lineGeo.setAttribute("position", new THREE.Float32BufferAttribute(lineVerts, 3));
    lineGeo.setAttribute("color", new THREE.Float32BufferAttribute(lineColors, 3));

    const lineMat = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.18,
      depthWrite: false,
    });
    const lines = new THREE.LineSegments(lineGeo, lineMat);
    scene.add(lines);

    // ── Mouse interaction ────────────────────────────────────────
    const mouse = { x: 0, y: 0 };
    const handleMouseMove = (e: MouseEvent) => {
      mouse.x = (e.clientX / window.innerWidth - 0.5) * 2;
      mouse.y = -(e.clientY / window.innerHeight - 0.5) * 2;
    };
    window.addEventListener("mousemove", handleMouseMove);

    // ── Resize ───────────────────────────────────────────────────
    const handleResize = () => {
      const w = el.clientWidth;
      const h = el.clientHeight;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    };
    window.addEventListener("resize", handleResize);

    // ── Animation loop ───────────────────────────────────────────
    let frame = 0;
    let raf: number;

    const animate = () => {
      raf = requestAnimationFrame(animate);
      frame++;

      // Slow rotation
      points.rotation.y += 0.0008;
      points.rotation.x += 0.0003;
      lines.rotation.y = points.rotation.y;
      lines.rotation.x = points.rotation.x;

      // Mouse parallax on camera
      camera.position.x += (mouse.x * 8 - camera.position.x) * 0.04;
      camera.position.y += (mouse.y * 6 - camera.position.y) * 0.04;
      camera.lookAt(scene.position);

      // Animate particle sizes for breathing effect
      const sizeAttr = geo.getAttribute("size") as THREE.BufferAttribute;
      for (let i = 0; i < COUNT; i += 4) {
        (sizeAttr.array as Float32Array)[i] = sizes[i] * (1 + 0.3 * Math.sin(frame * 0.02 + i * 0.1));
      }
      sizeAttr.needsUpdate = true;

      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("resize", handleResize);
      renderer.dispose();
      geo.dispose();
      mat.dispose();
      lineGeo.dispose();
      lineMat.dispose();
      if (el.contains(renderer.domElement)) el.removeChild(renderer.domElement);
    };
  }, []);

  return (
    <div
      ref={mountRef}
      className="absolute inset-0 w-full h-full"
      style={{ pointerEvents: "none" }}
    />
  );
}
