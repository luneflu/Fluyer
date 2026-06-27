// WGPU Renderer — femtovg cross-platform backend (non-Linux)
#![cfg(not(target_os = "linux"))]

use femtovg::{renderer::WGPURenderer, Canvas};
use std::sync::{Arc, Mutex};
use tauri::{async_runtime::block_on, Manager};
use tauri_plugin_device_info::DeviceInfoExt;
use wgpu::{BackendOptions, Backends, InstanceDescriptor, InstanceFlags};

pub fn create_surface(
    instance: &wgpu::Instance,
    app_handle: &tauri::AppHandle,
) -> Result<wgpu::Surface<'static>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    let surface = {
        use jni::objects::{JClass, JObject, JValue};
        use raw_window_handle::{
            AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
        };
        use std::ffi::c_void;
        use std::sync::mpsc;
        use tauri::Manager;

        let window = app_handle
            .get_webview_window("main")
            .ok_or("Failed to get main window")?;
        let (tx, rx) = mpsc::channel();

        window
            .with_webview(move |webview| {
                // jni_handle().exec provides the JNIEnv and Android Context natively
                webview
                    .jni_handle()
                    .exec(move |env, context, _android_webview| {
                        let result: Result<usize, String> =
                            (|| -> Result<usize, Box<dyn std::error::Error>> {
                                let class_context = env.find_class("android/content/Context")?;
                                let get_class_loader_method = env.get_method_id(
                                    class_context,
                                    "getClassLoader",
                                    "()Ljava/lang/ClassLoader;",
                                )?;

                                let class_loader = unsafe {
                                    env.call_method_unchecked(
                                        context,
                                        get_class_loader_method,
                                        jni::signature::ReturnType::Object,
                                        &[],
                                    )
                                }?
                                .l()?;

                                let class_class_loader = env.find_class("java/lang/ClassLoader")?;
                                let load_class_method = env.get_method_id(
                                    class_class_loader,
                                    "loadClass",
                                    "(Ljava/lang/String;)Ljava/lang/Class;",
                                )?;

                                let class_name_str =
                                    env.new_string("org.alvindimas05.fluyerplugin.FluyerPlugin")?;
                                let mut android_surface_obj: JObject = JObject::null();

                                crate::debug!("create_surface: Waiting for surface class load...");

                                loop {
                                    let fluyer_plugin_class_value = unsafe {
                                        env.call_method_unchecked(
                                            &class_loader,
                                            load_class_method,
                                            jni::signature::ReturnType::Object,
                                            &[JValue::Object(&class_name_str).as_jni()],
                                        )
                                    };

                                    if let Ok(val) = fluyer_plugin_class_value {
                                        let fluyer_plugin_class_obj = val.l()?;
                                        let fluyer_plugin_class: JClass =
                                            fluyer_plugin_class_obj.into();

                                        let field_id = env.get_static_field_id(
                                            &fluyer_plugin_class,
                                            "surface",
                                            "Landroid/view/Surface;",
                                        )?;

                                        let surface_obj_res = env.get_static_field_unchecked(
                                            &fluyer_plugin_class,
                                            field_id,
                                            jni::signature::JavaType::Object(
                                                "Landroid/view/Surface;".to_string(),
                                            ),
                                        );

                                        if let Ok(obj_val) = surface_obj_res {
                                            let obj = obj_val.l()?;
                                            if !obj.is_null() {
                                                crate::debug!(
                                                    "create_surface: Found valid surface object"
                                                );
                                                android_surface_obj = obj;
                                                break;
                                            }
                                        }
                                    }

                                    crate::debug!(
                                        "create_surface: Waiting for surface check iteration..."
                                    );
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }

                                if android_surface_obj.is_null() {
                                    return Err("Timed out waiting for Android Surface".into());
                                }

                                // Extract NativeWindow inside the JNI block
                                let native_window = unsafe {
                                    ndk::native_window::NativeWindow::from_surface(
                                        env.get_native_interface(),
                                        android_surface_obj.as_raw(),
                                    )
                                }
                                .ok_or("Failed to create native window from surface")?;

                                let native_window_ref = native_window.ptr().as_ptr();
                                std::mem::forget(native_window);

                                Ok(native_window_ref as usize)
                            })()
                            .map_err(|e| e.to_string());

                        let _ = tx.send(result);
                    });
            })
            .map_err(|e| e.to_string())?;

        // Block until the JNI closure completes and returns the pointer
        let native_window_ptr = rx
            .recv()
            .map_err(|_| "Channel closed before surface could be extracted".to_string())??
            as *mut c_void;

        let handle =
            AndroidNdkWindowHandle::new(std::ptr::NonNull::new(native_window_ptr).unwrap());
        let raw_window_handle = RawWindowHandle::AndroidNdk(handle);

        let display_handle = AndroidDisplayHandle::new();
        let raw_display_handle = RawDisplayHandle::Android(display_handle);

        unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })?
        }
    };

    #[cfg(not(target_os = "android"))]
    let surface = {
        let window = app_handle.get_webview_window("main").unwrap();
        instance.create_surface(window.clone())?
    };

    let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
    Ok(surface)
}
pub struct RendererState {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    canvas: Canvas<WGPURenderer>,
}

pub struct SharedRenderer {
    pub state: Mutex<Option<RendererState>>,
}

unsafe impl Send for RendererState {}
unsafe impl Sync for RendererState {}

pub fn setup_wgpu(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    crate::debug!("setup_wgpu: Starting femtovg WGPU initialization");

    let shared = Arc::new(SharedRenderer {
        state: Mutex::new(None),
    });
    app.manage(shared.clone());

    let app_handle = app.handle().clone();
    let window = app_handle.get_webview_window("main").unwrap();

    #[cfg(target_os = "android")]
    let size = if let Ok(Some(monitor)) = window.current_monitor() {
        *monitor.size()
    } else {
        window
            .inner_size()
            .unwrap_or(tauri::PhysicalSize::new(0, 0))
    };
    #[cfg(not(target_os = "android"))]
    let size = window
        .inner_size()
        .unwrap_or(tauri::PhysicalSize::new(1280, 720));

    crate::debug!("setup_wgpu: Window size {}x{}", size.width, size.height);

    #[cfg(not(target_os = "macos"))]
    let backends = Backends::GL;
    #[cfg(target_os = "macos")]
    let backends = Backends::METAL;

    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends,
        ..InstanceDescriptor::new_without_display_handle()
    });

    #[cfg(not(target_os = "android"))]
    let surface = create_surface(&instance, &app_handle)?;

    std::thread::spawn(move || {
        #[cfg(target_os = "android")]
        let surface = match create_surface(&instance, &app_handle) {
            Ok(s) => s,
            Err(e) => {
                crate::error!("Failed to create surface: {}", e);
                return;
            }
        };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits:
                wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            memory_hints: wgpu::MemoryHints::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        }))
        .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0].remove_srgb_suffix();

        let alpha_mode = swapchain_capabilities
            .alpha_modes
            .iter()
            .find(|&&m| m == wgpu::CompositeAlphaMode::PreMultiplied)
            .or_else(|| {
                swapchain_capabilities
                    .alpha_modes
                    .iter()
                    .find(|&&m| m == wgpu::CompositeAlphaMode::PostMultiplied)
            })
            .copied()
            .unwrap_or(swapchain_capabilities.alpha_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: if size.width > 0 { size.width } else { 1 },
            height: if size.height > 0 { size.height } else { 1 },
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Build femtovg WGPU renderer — takes owned Device + Queue
        let renderer = WGPURenderer::new(device.clone(), queue.clone());
        let mut canvas = Canvas::new(renderer).expect("Cannot create femtovg canvas");
        canvas.set_size(config.width, config.height, 1.0);

        let mut state_guard = shared.state.lock().unwrap();
        *state_guard = Some(RendererState {
            instance,
            surface: Some(surface),
            device,
            queue,
            config,
            canvas,
        });

        crate::debug!("setup_wgpu: WGPU async initialization complete");
    });

    Ok(())
}

pub fn handle_wgpu_resize(app_handle: &tauri::AppHandle, width: u32, height: u32) {
    if let Some(shared) = app_handle.try_state::<Arc<SharedRenderer>>() {
        let mut state_guard = shared.state.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            state.config.width = if width > 0 { width } else { 1 };
            state.config.height = if height > 0 { height } else { 1 };
            if let Some(surface) = &state.surface {
                surface.configure(&state.device, &state.config);
            }
            let (w, h) = (state.config.width, state.config.height);
            state.canvas.set_size(w, h, 1.0);
        }
    }
}

pub fn suspend_wgpu(app_handle: &tauri::AppHandle) {
    #[cfg(target_os = "android")]
    {
        crate::debug!("Suspending WGPU");
        if let Some(shared) = app_handle.try_state::<Arc<SharedRenderer>>() {
            let mut state_guard = shared.state.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                state.surface = None;
            }
        }
    }
}

pub fn resume_wgpu(app_handle: &tauri::AppHandle) {
    #[cfg(target_os = "android")]
    {
        crate::debug!("Resuming WGPU logic");
        std::thread::spawn({
            let app_handle = app_handle.clone();
            move || {
                if let Some(shared) = app_handle.try_state::<Arc<SharedRenderer>>() {
                    let has_surface = {
                        let state_guard = shared.state.lock().unwrap();
                        state_guard.as_ref().map_or(false, |s| s.surface.is_some())
                    };

                    if !has_surface {
                        crate::debug!("Resuming WGPU: Recreating surface");
                        let instance = {
                            let state_guard = shared.state.lock().unwrap();
                            if let Some(state) = state_guard.as_ref() {
                                state.instance.clone()
                            } else {
                                return;
                            }
                        };

                        match create_surface(&instance, &app_handle) {
                            Ok(surface) => {
                                let mut state_guard = shared.state.lock().unwrap();
                                if let Some(state) = state_guard.as_mut() {
                                    surface.configure(&state.device, &state.config);
                                    state.surface = Some(surface);
                                    crate::debug!("Resuming WGPU: Surface recreated");
                                    crate::renderer::trigger_redraw();
                                }
                            }
                            Err(e) => {
                                crate::error!("Failed to recreate surface on resume: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }
}

pub fn start_render_loop(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let shared = match app_handle.try_state::<Arc<SharedRenderer>>() {
            Some(s) => s,
            None => {
                crate::error!("start_render_loop: RendererState not found");
                return;
            }
        };

        let refresh_rate = app_handle
            .device_info()
            .get_display_info()
            .unwrap()
            .refresh_rate
            .unwrap();

        let global = match app_handle.try_state::<Arc<crate::renderer::GlobalRenderer>>() {
            Some(g) => g,
            None => return,
        };

        loop {
            let mut bg_state = global.bg_state.lock().unwrap();

            // Wait until something needs rendering
            while bg_state.transition_start.is_none() && !bg_state.needs_redraw {
                bg_state = global.cond.wait(bg_state).unwrap();
            }

            bg_state.needs_redraw = false;

            let mut state_guard = shared.state.lock().unwrap();
            let state = match state_guard.as_mut() {
                Some(s) => s,
                None => {
                    drop(state_guard);
                    drop(bg_state);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
            };

            // Draw with femtovg
            crate::renderer::draw_background(&mut state.canvas, &mut bg_state);

            // Get surface frame
            let surface = match &state.surface {
                Some(s) => s,
                None => {
                    drop(state_guard);
                    drop(bg_state);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
            };

            let frame = match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(f) => f,
                _ => {
                    crate::warn!("Failed to get current texture");
                    drop(state_guard);
                    drop(bg_state);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    continue;
                }
            };

            // flush_to_output accepts &wgpu::Texture via WGPURenderOutput::from
            let cmd_buf = state.canvas.flush_to_output(&frame.texture);
            if let Some(cmd) = cmd_buf {
                state.queue.submit(std::iter::once(cmd));
            }
            frame.present();

            drop(state_guard);
            drop(bg_state);
            std::thread::sleep(std::time::Duration::from_millis(
                (1000.0 / refresh_rate) as u64,
            ));
        }
    });
}
