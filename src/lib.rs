pub mod data;
pub mod vector;
pub mod text;
pub mod app;
pub mod pipeline;

pub mod camera;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};


// ====== application build ======

pub async fn build<E: app::Application>(title: &str) -> app::AppSkeleton {
    // required by wgpu
    env_logger::init();

    // create window
    let (window, event_loop) = create_window(title);

    // get GPU handle
    let instance = get_gpu_instance();

    // create surface
    let surface = create_surface(&window, &instance).unwrap();

    // request adapter
    let (adapter, device, queue) = request_adapter::<E>(&instance, &surface).await;

    let size = window.inner_size();
    let screen_size = data::ScreenSize::new(size.width, size.height);
    let config = create_surface_configuration(&surface, &adapter, &size);
    surface.configure(&device, &config);

    app::AppSkeleton {
        window,
        event_loop,
        _instance: instance,
        surface,
        adapter: adapter,
        device,
        queue,
        config,
        screen_size,
    }
}

fn create_window(title: &str) -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    (
        winit::window::WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap(),
        event_loop
    )
}

fn get_gpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all),
        dx12_shader_compiler: Default::default(),
    })
}

fn create_surface(window: &Window, instance: &wgpu::Instance) ->
    Result<wgpu::Surface, wgpu::CreateSurfaceError>
{
    unsafe {
        instance.create_surface(window)
    }
}

async fn request_adapter<E: app::Application>(instance: &wgpu::Instance, surface: &wgpu::Surface) ->
    (wgpu::Adapter, wgpu::Device, wgpu::Queue)
{
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    ).await.unwrap();
    
    // print adapter info
    let adapter_info = adapter.get_info();
    println!("Attempting adapter: {} ({:?})", adapter_info.name, adapter_info.backend);

    let required_features = E::required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features: {:?}",
        required_features - adapter_features
    );

    let required_downlevel_capabilities = E::required_downlevel_capabilities();
    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    assert!(
        downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
        "Adapter does not support the minimum shader model required: {:?}",
        required_downlevel_capabilities.shader_model
    );
    assert!(
        downlevel_capabilities.flags.contains(required_downlevel_capabilities.flags),
        "Adapter does not support the downlevel capabilities required: {:?}",
        required_downlevel_capabilities.flags - downlevel_capabilities.flags
    );

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: (E::optional_features() & adapter.features()) | E::required_features(),
            limits: E::required_limits().using_resolution(adapter.limits()),
            label: None,
        },
        trace_dir.ok().as_ref().map(std::path::Path::new),
    ).await
    .expect("Adapter device creation failed!");

    (adapter, device, queue)
}

fn create_surface_configuration(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
    size: &PhysicalSize<u32>
) -> wgpu::SurfaceConfiguration {
    let surface_capabilities = surface.get_capabilities(&adapter);

    let surface_format = surface_capabilities.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_capabilities.formats[0]);

    // attempt to use Mailbox present_mode, otherwise fallback to Fifo (VSync)
    println!("avail present_modes: {:?}", surface_capabilities.present_modes);
    let surface_present_mode = surface_capabilities.present_modes.iter()
        .copied()
        .find(|m| m == &wgpu::PresentMode::Mailbox)
        .unwrap_or(wgpu::PresentMode::Fifo);
    println!("using present_mode: {:?}", surface_present_mode);

    let surface_view_format = surface_format.add_srgb_suffix();

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_present_mode,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![surface_view_format],
    }
}

// ====== applicaiton run ======

pub fn run<E: app::Application>(mut app: E, mut skeleton: app::AppSkeleton) {
    log::info!("Entering event loop...");
    skeleton.event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == skeleton.window.id() => if !app.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(pysical_size) => {
                        if resize_possible(pysical_size) {
                            //size = *pysical_size;
                            skeleton.config.width = pysical_size.width;
                            skeleton.config.height = pysical_size.height;
                            skeleton.surface.configure(&skeleton.device, &skeleton.config);
                        }
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // gotta deference it twice cause apparently its &&mut
                        if resize_possible(*&new_inner_size) {
                            //size = **new_inner_size;
                            skeleton.config.width = new_inner_size.width;
                            skeleton.config.height = new_inner_size.height;
                            skeleton.surface.configure(&skeleton.device, &skeleton.config);
                        }
                    },
                    _ => {}
                }
            },
            Event::RedrawRequested(window_id) if window_id == skeleton.window.id() => {
                app.update(&skeleton.queue);
                match app.render(&skeleton.surface, &skeleton.device, &skeleton.queue) {
                    Ok(_) => {},
                    // reconfigure surface if lost
                    Err(wgpu::SurfaceError::Lost) => skeleton.surface.configure(&skeleton.device, &skeleton.config),
                    // out of memory, attempt reset
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // ignore other errors, they'll probably be gone by the next frame
                    Err(e) => eprintln!("{:?}", e),
                };
            },
            Event::RedrawEventsCleared => {
                // manually trigger RedrawRequested otherwise it only triggers once
                skeleton.window.request_redraw();
            }
            _ => {}
        }
    });
}

fn resize_possible(size: &PhysicalSize<u32>) -> bool {
    size.width > 0 && size.height > 0
}
