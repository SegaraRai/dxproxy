# DXProxy

A DirectX API proxying **template** written in Rust that provides transparent interception and logging of DirectX calls. This project serves as a **customizable foundation** for creating tools that modify application behavior at the graphics API level.

## Overview

DXProxy acts as a drop-in replacement for DirectX DLLs, intercepting graphics API calls and wrapping them with proxy objects. Currently, it supports **DirectX 9 (Direct3D 9)** with logging capabilities and is designed to be easily extensible for adding custom functionality.

**🎯 This is a template project** - fork it and modify the code to add your own features like performance analysis, shader modification, texture replacement, or custom rendering effects.

## Features

- **Transparent Proxying**: Drop-in replacement for d3d9.dll with full API compatibility
- **Comprehensive Logging**: Detailed tracing of all DirectX API calls with configurable output
- **COM Interface Wrapping**: Complete proxy implementation for all Direct3D 9 interfaces
- **Template Architecture**: Ready-to-modify foundation for custom DirectX tools
- **Modern Rust Implementation**: Memory-safe implementation using the windows-rs crate
- **Extensible Design**: Easy to add hooks, filters, and custom behavior

## Use Cases & Examples

This template can be customized for various purposes:

- 🎮 **Game Modifications**: Texture replacement, shader injection, UI overlays
- 📊 **Performance Analysis**: Frame time monitoring, draw call counting, resource tracking
- 🔧 **Development Tools**: Graphics debugging, API call visualization
- 🎨 **Visual Enhancement**: Post-processing effects, upscaling, color correction
- 🔍 **Research**: Graphics API behavior analysis, compatibility testing

## Quick Start

### 1. Fork & Clone

```bash
git clone <your-fork-url>
cd dxproxy
```

### 2. Build

```bash
cargo build --release

# or, for 32-bit applications:
cargo build --release --target=i686-pc-windows-msvc
```

### 3. Deploy

Copy `target/release/d3d9.dll` (or `target/i686-pc-windows-msvc/release/d3d9.dll` for 32-bit) to the directory of your target application.

### 4. Customize

Start modifying the proxy implementations in `core/src/dx9/com/` to add your features!

## Customization Guide

### Adding Custom Logic

The template is designed for easy modification. Here are common customization patterns:

#### Example 1: Texture Replacement

```rust
// In IDirect3DDevice9_Impl::SetTexture
fn SetTexture(&self, stage: u32, ptexture: Ref<IDirect3DTexture9>) -> Result<()> {
    // Your custom logic here
    if let Some(replacement) = self.get_texture_replacement(&ptexture) {
        return unsafe { self.target.SetTexture(stage, replacement) };
    }

    // Default behavior
    let target = self.get_context().get_target_nullable(ptexture);
    unsafe { self.target.SetTexture(stage, target) }
}
```

#### Example 2: Performance Monitoring

```rust
// In IDirect3DDevice9_Impl::Present
fn Present(&self, /* ... */) -> Result<()> {
    let start_time = std::time::Instant::now();

    let result = unsafe { self.target.Present(/* ... */) };

    let frame_time = start_time.elapsed();
    self.performance_tracker.record_frame(frame_time);

    result
}
```

#### Example 3: Shader Modification

```rust
// In IDirect3DDevice9_Impl::CreatePixelShader
fn CreatePixelShader(&self, pfunction: *const u32, ppshader: *mut Ref<IDirect3DPixelShader9>) -> Result<()> {
    // Modify shader bytecode before creation
    let modified_bytecode = self.shader_patcher.patch_shader(pfunction);

    unsafe { self.target.CreatePixelShader(modified_bytecode.as_ptr(), ppshader) }
}
```

### Configuration System

Extend the configuration in `core/src/dx9/config.rs`:

```rust
#[derive(Debug, Clone)]
pub struct DX9ProxyConfig {
    pub logging: LoggingConfig,
    // Add your custom config fields
    pub enable_texture_replacement: bool,
    pub performance_monitoring: bool,
    pub shader_debugging: bool,
}
```

## Project Structure

```text
dxproxy/
├── core/                    # Core proxy library - MODIFY THIS
│   ├── src/
│   │   ├── dx9/             # DirectX 9 implementation
│   │   │   ├── com/         # COM interface proxies - ADD YOUR LOGIC HERE
│   │   │   ├── config.rs    # Configuration - EXTEND THIS
│   │   │   └── dll.rs       # DLL export functions
│   │   └── common/          # Shared utilities
│   └── Cargo.toml
├── entrypoints/             # DLL entry points - USUALLY NO CHANGES NEEDED
│   └── d3d9/
└── examples/                # Usage examples (add your own!)
```

## Supported APIs

- ✅ **DirectX 9 (d3d9.dll)** - Complete implementation
- 🔮 **DirectX 11** - Planned
- 🔮 **DirectX 12** - Planned
- 🔮 **Other DirectX APIs** - Extensible architecture

## DirectX 9 Implementation

The DirectX 9 proxy provides complete coverage of the Direct3D 9 API:

### Supported Interfaces

- **Core Interfaces**:
  - `IDirect3D9` / `IDirect3D9Ex` - Main Direct3D object
  - `IDirect3DDevice9` / `IDirect3DDevice9Ex` - Rendering device
- **Resource Interfaces**:
  - `IDirect3DSurface9` - 2D surfaces
  - `IDirect3DTexture9` - 2D textures
  - `IDirect3DCubeTexture9` - Cube textures
  - `IDirect3DVolumeTexture9` - Volume textures
  - `IDirect3DVertexBuffer9` - Vertex buffers
  - `IDirect3DIndexBuffer9` - Index buffers
- **Pipeline Interfaces**:
  - `IDirect3DVertexShader9` - Vertex shaders
  - `IDirect3DPixelShader9` - Pixel shaders
  - `IDirect3DStateBlock9` - State blocks
- **Presentation Interfaces**:
  - `IDirect3DSwapChain9` / `IDirect3DSwapChain9Ex` - Swap chains
  - `IDirect3DQuery9` - GPU queries

### Key Features

- **Transparent Passthrough**: All calls are forwarded to the original DirectX implementation
- **Comprehensive Logging**: Every API call is logged with parameters and return values
- **State Tracking**: Maintains proxy objects for resource lifecycle management
- **Error Handling**: Proper error propagation and logging

## Development

### Testing

The project includes unit and integration tests to verify proxy behavior:

```bash
# Run all tests
cargo test

# Run a specific test
cargo test common::try_out_param::tests::test_try_out_param_success

# Run with logging enabled
RUST_LOG=debug cargo test -- --nocapture
```

See `core/src/tests/README.md` for more details about the testing approach.

### Adding Custom Functionality

The proxy system is designed to be easily extensible. To add custom behavior:

1. **Modify COM Proxies**: Edit the relevant proxy implementation in `core/src/dx9/com/`
2. **Add Configuration**: Extend `DX9ProxyConfig` in `config.rs`
3. **Implement Logic**: Add your custom logic before/after forwarding calls to the target

Example - Adding texture filtering:

```rust
// In IDirect3DDevice9_Impl
fn SetTexture(&self, stage: u32, ptexture: Ref<IDirect3DTexture9>) -> Result<()> {
    // Custom logic before
    tracing::info!("Setting texture at stage {stage}");

    // Forward to original
    let target = self.get_context().get_target_nullable(ptexture);
    unsafe { self.target.SetTexture(stage, target) }
}
```

### Architecture

- **Workspace Structure**: Multi-crate workspace with core library and DLL entry points
- **COM Proxying**: Windows-rs based COM interface implementations
- **Logging**: Structured logging with tracing crate
- **Configuration**: Environment-based configuration system

## Technical Details

### COM Interface Proxying

Each DirectX interface is wrapped by a corresponding proxy struct that:

- Implements the same COM interface using `#[implement()]` macro
- Maintains a reference to the original target interface
- Forwards all method calls to the target
- Provides logging and instrumentation

### DLL Replacement

The proxy DLL:

- Exports the same functions as the original d3d9.dll
- Loads the original system DLL at runtime
- Intercepts creation functions (`Direct3DCreate9`, `Direct3DCreate9Ex`)
- Returns proxy-wrapped objects instead of originals

### Memory Management

- Uses Rust's ownership system for memory safety
- Proper COM reference counting via windows-rs
- Automatic cleanup of proxy objects on drop

## Development Workflow

### 1. Identify Your Target

- What DirectX calls do you want to intercept?
- What modifications do you want to make?

### 2. Find the Right Interface

- Check `core/src/dx9/com/` for the relevant COM interface proxy
- Most rendering calls go through `IDirect3DDevice9`

### 3. Implement Your Logic

- Add custom behavior before/after the original call
- Use the existing logging patterns for debugging

### 4. Test & Iterate

- Test with your target application
- Use the logging output to verify your changes

### 5. Share Your Work

- Consider contributing improvements back to the template
- Share your customized version with the community

## Contributing to the Template

We welcome contributions that improve the template's foundation:

- **Bug Fixes**: Fix issues in the base proxy implementation
- **API Coverage**: Add support for missing DirectX interfaces
- **Documentation**: Improve guides and examples
- **Architecture**: Enhance the extensibility framework
- **Examples**: Add common customization examples

**Note**: This is a template project. Major feature additions should be implemented as examples or extensions rather than core functionality.

## FAQ

**Q: Can I use this in production?**
A: This is a template for development. Thoroughly test any customizations before production use.

**Q: Will this work with anti-cheat systems?**
A: DLL replacement may trigger anti-cheat systems. Use responsibly and only with applications you own or have permission to modify.

**Q: How do I add support for DirectX 11/12?**
A: The architecture is extensible. Follow the DirectX 9 implementation pattern in the `dx9` module as a reference.

**Q: Can I distribute my modified version?**
A: Yes, but ensure you comply with the license terms and any applicable software licenses.

## License

This project is licensed under the [MIT License](LICENSE). You are free to use, modify, and distribute this code as long as you include the original license and copyright notice.

## Disclaimer

This software is for educational and development purposes. Ensure you comply with the license terms of any applications you modify and respect intellectual property rights.
