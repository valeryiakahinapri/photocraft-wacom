import type { StorybookConfig } from "@storybook/react-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.@(ts|tsx)"],
  addons: ["storybook-annotakit"],
  framework: { name: "@storybook/react-vite", options: {} },
  // The hub's postcss.config.mjs (Tailwind 4 for Next) is not a valid Vite plugin
  // chain — stories carry their own inline styles, so drop PostCSS entirely here.
  viteFinal: async (config) => {
    config.css = { ...config.css, postcss: { plugins: [] } };
    return config;
  },
};

export default config;
