"use client";

import { useEffect, useRef } from "react";

type Particle = {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  hue: number;
};

const BASE_PARTICLE_COUNT = 56;
const MOBILE_PARTICLE_COUNT = 28;
const MAX_CONNECTION_DISTANCE = 140;
const MOBILE_CONNECTION_DISTANCE = 92;
const TARGET_FPS = 30;

export default function NeuralCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;

    let width = 0;
    let height = 0;
    let raf = 0;
    let lastFrame = 0;
    let running = true;
    let pointerX = 0;
    let pointerY = 0;
    let pointerActive = false;
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const particles: Particle[] = [];

    const configureCanvas = () => {
      const rect = canvas.getBoundingClientRect();
      const nextWidth = Math.max(1, Math.floor(rect.width));
      const nextHeight = Math.max(1, Math.floor(rect.height));
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);

      width = nextWidth;
      height = nextHeight;
      canvas.width = Math.floor(nextWidth * dpr);
      canvas.height = Math.floor(nextHeight * dpr);
      canvas.style.width = `${nextWidth}px`;
      canvas.style.height = `${nextHeight}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    const resetParticles = () => {
      particles.length = 0;
      const isCompact = width < 768;
      const count = reducedMotion
        ? Math.max(16, Math.round((isCompact ? MOBILE_PARTICLE_COUNT : BASE_PARTICLE_COUNT) * 0.55))
        : isCompact
          ? MOBILE_PARTICLE_COUNT
          : BASE_PARTICLE_COUNT;

      for (let i = 0; i < count; i += 1) {
        const speedScale = reducedMotion ? 0.06 : isCompact ? 0.1 : 0.14;
        particles.push({
          x: Math.random() * width,
          y: Math.random() * height,
          vx: (Math.random() - 0.5) * speedScale,
          vy: (Math.random() - 0.5) * speedScale,
          radius: Math.random() * 1.7 + 1,
          hue: 210 + Math.random() * 55,
        });
      }
    };

    const drawBackgroundGlow = () => {
      const gradient = ctx.createRadialGradient(
        width * 0.52,
        height * 0.38,
        0,
        width * 0.52,
        height * 0.38,
        Math.max(width * 0.42, height * 0.55),
      );
      gradient.addColorStop(0, "rgba(99, 102, 241, 0.11)");
      gradient.addColorStop(0.45, "rgba(59, 130, 246, 0.05)");
      gradient.addColorStop(1, "rgba(2, 6, 23, 0)");

      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, width, height);
    };

    const drawConnections = (isCompact: boolean) => {
      const maxDistance = isCompact ? MOBILE_CONNECTION_DISTANCE : MAX_CONNECTION_DISTANCE;
      const maxDistanceSq = maxDistance * maxDistance;

      for (let i = 0; i < particles.length; i += 1) {
        const a = particles[i];

        for (let j = i + 1; j < particles.length; j += 1) {
          const b = particles[j];
          const dx = a.x - b.x;
          const dy = a.y - b.y;
          const distanceSq = dx * dx + dy * dy;

          if (distanceSq > maxDistanceSq) continue;

          const alpha = 1 - distanceSq / maxDistanceSq;
          ctx.strokeStyle = `rgba(125, 211, 252, ${alpha * (isCompact ? 0.08 : 0.12)})`;
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      }
    };

    const drawParticles = () => {
      for (const particle of particles) {
        ctx.beginPath();
        ctx.fillStyle = `hsla(${particle.hue}, 90%, 72%, 0.9)`;
        ctx.shadowColor = `hsla(${particle.hue}, 90%, 70%, 0.32)`;
        ctx.shadowBlur = 12;
        ctx.arc(particle.x, particle.y, particle.radius, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.shadowBlur = 0;
    };

    const updateParticles = (isCompact: boolean) => {
      const pointerForce = reducedMotion ? 0 : isCompact ? 24 : 34;

      for (const particle of particles) {
        if (pointerActive) {
          const dx = pointerX - particle.x;
          const dy = pointerY - particle.y;
          const distance = Math.sqrt(dx * dx + dy * dy) || 1;

          if (distance < pointerForce) {
            const influence = 1 - distance / pointerForce;
            particle.vx -= (dx / distance) * influence * 0.006;
            particle.vy -= (dy / distance) * influence * 0.006;
          }
        }

        particle.x += particle.vx;
        particle.y += particle.vy;
        particle.vx *= 0.992;
        particle.vy *= 0.992;

        if (particle.x < -20) particle.x = width + 20;
        if (particle.x > width + 20) particle.x = -20;
        if (particle.y < -20) particle.y = height + 20;
        if (particle.y > height + 20) particle.y = -20;
      }
    };

    const render = (time: number) => {
      if (!running) return;

      raf = window.requestAnimationFrame(render);

      if (!reducedMotion && time - lastFrame < 1000 / TARGET_FPS) {
        return;
      }

      lastFrame = time;
      const isCompact = width < 768;

      ctx.clearRect(0, 0, width, height);
      drawBackgroundGlow();
      updateParticles(isCompact);
      drawConnections(isCompact);
      drawParticles();
    };

    const handleResize = () => {
      configureCanvas();
      resetParticles();
    };

    const handlePointerMove = (event: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      pointerX = event.clientX - rect.left;
      pointerY = event.clientY - rect.top;
      pointerActive =
        pointerX >= 0 &&
        pointerX <= rect.width &&
        pointerY >= 0 &&
        pointerY <= rect.height;
    };

    const handlePointerLeave = () => {
      pointerActive = false;
    };

    const handleVisibilityChange = () => {
      running = document.visibilityState === "visible";

      if (running) {
        lastFrame = 0;
        raf = window.requestAnimationFrame(render);
      } else {
        window.cancelAnimationFrame(raf);
      }
    };

    configureCanvas();
    resetParticles();
    raf = window.requestAnimationFrame(render);

    window.addEventListener("resize", handleResize);
    window.addEventListener("pointermove", handlePointerMove, { passive: true });
    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("pointerleave", handlePointerLeave);

    return () => {
      running = false;
      window.cancelAnimationFrame(raf);
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerleave", handlePointerLeave);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="absolute inset-0 h-full w-full"
      style={{ pointerEvents: "none" }}
    />
  );
}
