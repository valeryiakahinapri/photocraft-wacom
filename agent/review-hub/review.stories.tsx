import { useEffect, useRef } from "react";
import type { Meta, StoryObj } from "@storybook/react";

/**
 * The photocraft web app (Rust + egui + WebGPU, baked to /photocraft/) mounted
 * DIRECTLY in the story canvas — one DOM with the Annotakit layer, so region
 * pins and element pins work over the live app instead of dying inside an
 * iframe (pointer events never bubble out of a nested iframe).
 *
 * The app boots against the same elements its own index.html provides
 * (`#photocraft_canvas` + `#photocraft_loading`, see apps/photocraft-web's
 * web.rs); the hashed asset names are read from the baked index.html so every
 * re-bake just works.
 */
const PhotocraftInCanvas = ({ height }: { height: string }) => {
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = host.current;
    if (!el) return;
    let cancelled = false;

    // The app cannot boot twice (winit takes over the canvas); a story remount
    // adopts the live canvas instead of re-initializing it.
    const existing = document.getElementById("photocraft_canvas");
    if (existing) {
      el.appendChild(existing);
      el.appendChild(document.getElementById("photocraft_loading") ?? document.createComment(""));
      return;
    }

    (async () => {
      const page = await (await fetch("/photocraft/")).text();
      const js = page.match(/href="\.\/(photocraft-web-[^"]+\.js)"/)?.[1];
      if (!js || cancelled) return;
      const mod = await import(/* @vite-ignore */ `/photocraft/${js}`);
      if (cancelled) return;
      const canvas = document.createElement("canvas");
      canvas.id = "photocraft_canvas";
      Object.assign(canvas.style, {
        position: "absolute",
        inset: "0",
        width: "100%",
        height: "100%",
        display: "block",
      });
      const loading = document.createElement("div");
      loading.id = "photocraft_loading";
      Object.assign(loading.style, {
        position: "absolute",
        inset: "0",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        color: "#b8b8b8",
        font: "14px system-ui, sans-serif",
        pointerEvents: "none",
      });
      loading.innerHTML = "<p>Loading PhotoCraft…</p>";
      el.style.position = "relative";
      el.style.height = height;
      el.appendChild(canvas);
      el.appendChild(loading);
      const wasm = `/photocraft/${js.replace(/\.js$/, "_bg.wasm")}`;
      await mod.default({ module_or_path: wasm });
    })();

    return () => {
      cancelled = true;
    };
  }, [height]);

  return <div ref={host} style={{ width: "100%", height }} />;
};

const meta: Meta<typeof PhotocraftInCanvas> = {
  title: "Review/Photocraft",
  component: PhotocraftInCanvas,
  parameters: { layout: "fullscreen" },
};

export default meta;

export const App: StoryObj<typeof PhotocraftInCanvas> = {
  args: { height: "100vh" },
};
