"use client";
import { useEffect, useRef } from "react";
import * as THREE from "three";

export default function NeuralCanvas() {
  const mountRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = mountRef.current;
    if (!el) return;

    const W = el.clientWidth;
    const H = el.clientHeight;
    if (!W || !H) return;

    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const compactMode = W < 768;
    const particleCount = reducedMotion ? 120 : compactMode ? 180 : W < 1200 ? 260 : 360;
    const connectionThreshold = reducedMotion ? 0 : compactMode ? 12 : W < 1200 ? 14 : 16;
    const maxConnections = compactMode ? 90 : W < 1200 ? 170 : 260;
    const breatheStride = compactMode ? 10 : 6;
    const probe = document.createElement("canvas");
    const hasWebgl =
      !!window.WebGLRenderingContext &&
      !!(probe.getContext("webgl") || probe.getContext("experimental-webgl"));
    if (!hasWebgl) return;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(60, W / H, 0.1, 2000);
    camera.position.set(0, 0, compactMode ? 140 : 128);

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({
        antialias: !compactMode,
        alpha: true,
        powerPreference: "low-power",
      });
    } catch {
      return;
    }
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(W, H);
    renderer.setClearColor(0x000000, 0);
    el.appendChild(renderer.domElement);

    // Keep the backdrop light enough that first paint and interaction stay smooth.
    const COUNT = particleCount;
    const positions = new Float32Array(COUNT * 3);
    const colors = new Float32Array(COUNT * 3);
    const sizes = new Float32Array(COUNT);

    const palette = [
      new THREE.Color("#6366f1"),
      new THREE.Color("#818cf8"),
      new THREE.Color("#93c5fd"),
      new THREE.Color("#22d3ee"),
    ];

    for (let i = 0; i < COUNT; i++) {
      const r = 88 + Math.random() * 44;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      positions[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta) * 0.6;
      positions[i * 3 + 2] = r * Math.cos(phi);

      const c = palette[Math.floor(Math.random() * palette.length)];
      colors[i * 3] = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;

      sizes[i] = Math.random() * 1.75 + 0.7;
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

    const lineGeo = new THREE.BufferGeometry();
    const lineVerts: number[] = [];
    const lineColors: number[] = [];
    let builtConnections = 0;

    for (let i = 0; i < COUNT && builtConnections < maxConnections; i++) {
      for (let j = i + 1; j < COUNT && builtConnections < maxConnections; j++) {
        const dx = positions[i * 3] - positions[j * 3];
        const dy = positions[i * 3 + 1] - positions[j * 3 + 1];
        const dz = positions[i * 3 + 2] - positions[j * 3 + 2];
        const dist = Math.sqrt(dx*dx + dy*dy + dz*dz);
        if (dist < connectionThreshold) {
          const alpha = 1 - dist / connectionThreshold;
          lineVerts.push(
            positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2],
            positions[j * 3], positions[j * 3 + 1], positions[j * 3 + 2],
          );
          const r = 0.44 + alpha * 0.1;
          const g = 0.60 + alpha * 0.12;
          const b = 0.95;
          lineColors.push(r, g, b, r, g, b);
          builtConnections += 1;
        }
      }
    }

    lineGeo.setAttribute("position", new THREE.Float32BufferAttribute(lineVerts, 3));
    lineGeo.setAttribute("color", new THREE.Float32BufferAttribute(lineColors, 3));

    const lineMat = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: compactMode ? 0.08 : 0.12,
      depthWrite: false,
    });
    const lines = new THREE.LineSegments(lineGeo, lineMat);
    scene.add(lines);

    const mouse = { x: 0, y: 0 };
    const handleMouseMove = (e: MouseEvent) => {
      mouse.x = (e.clientX / window.innerWidth - 0.5) * 2;
      mouse.y = -(e.clientY / window.innerHeight - 0.5) * 2;
    };
    if (!reducedMotion && !compactMode) {
      window.addEventListener("mousemove", handleMouseMove);
    }

    const handleResize = () => {
      const w = el.clientWidth;
      const h = el.clientHeight;
      if (!w || !h) return;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    };
    window.addEventListener("resize", handleResize);

    let frame = 0;
    let raf: number;

    const animate = () => {
      raf = requestAnimationFrame(animate);
      frame++;

      points.rotation.y += reducedMotion ? 0.0001 : compactMode ? 0.0002 : 0.00035;
      points.rotation.x += reducedMotion ? 0.00005 : 0.00014;
      lines.rotation.y = points.rotation.y;
      lines.rotation.x = points.rotation.x;

      camera.position.x += (mouse.x * 4 - camera.position.x) * 0.035;
      camera.position.y += (mouse.y * 3 - camera.position.y) * 0.035;
      camera.lookAt(scene.position);

      const sizeAttr = geo.getAttribute("size") as THREE.BufferAttribute;
      for (let i = 0; i < COUNT; i += breatheStride) {
        (sizeAttr.array as Float32Array)[i] =
          sizes[i] * (1 + 0.16 * Math.sin(frame * 0.012 + i * 0.06));
      }
      sizeAttr.needsUpdate = true;

      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(raf);
      if (!reducedMotion && !compactMode) {
        window.removeEventListener("mousemove", handleMouseMove);
      }
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
