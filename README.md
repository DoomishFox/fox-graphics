# fox-graphics

WGPU drawing primitives used by FoxCAD. Version 0.2 replaces the earlier application/window/camera prototype. It does not preserve fig compatibility.

The caller owns the WGPU device, queue, surface, event loop and render pass. `StrokeRenderer` accepts logical-pixel strokes and a viewport size, uploads triangle geometry, and draws into the caller's pass. It has no egui or winit dependency. The initial renderer supports independent antialiased straight strokes; joins, dashes, fills, curves and drawing text remain future work.

World geometry and f64 camera math belong to the caller. Clip and transform world coordinates before sending f32 screen-space primitives.

The FoxCAD Phase 0 application is the integration example. The old application, camera and atlas prototype modules have been removed.
