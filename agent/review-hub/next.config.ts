import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  typescript: {
    ignoreBuildErrors: true,
  },
  reactStrictMode: false,
  async rewrites() {
    return [
      // Static bakes under public/ have no directory-index — without these the
      // bare paths 404 and Next's trailing-slash redirect would break relative refs.
      { source: "/photocraft", destination: "/photocraft/index.html" },
      { source: "/storybook", destination: "/storybook/index.html" },
    ];
  },
};

export default nextConfig;
