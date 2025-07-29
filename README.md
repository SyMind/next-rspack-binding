# next-rspack-binding

> This is an experimental project

After integrating Rspack into Next.js, we discovered that the primary performance bottleneck stems from frequent communication between JavaScript and Rust. Next.js contains complex logic in Rspack configurations and plugins that require frequent calls to Rspack's exposed JavaScript APIs.

The goal of this project is to migrate these performance-critical logic components to Rust, thereby eliminating JavaScript-Rust communication overhead and significantly improving Next.js performance when using Rspack.

## Currently Rust-ported Logic

### rspackConfig.externals

Next.js has complex externals configuration where each module requires calling Next.js's configured externals function, creating a major performance bottleneck for Rspack.
