use std::{cell::RefCell, collections::HashMap, error::Error, sync::Arc, time::Duration};

use ash::vk::SurfaceKHR;
use fent_ecs::prelude::*;
use stellar_window::{
    NoRenderer, WindowResized,
    glfw::{GlfwWindowId, framebuffer_size, required_vulkan_extensions, with_window},
};
use vulkano::{
    Handle, Validated, Version, VulkanError, VulkanLibrary, VulkanObject,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderingAttachmentInfo, RenderingInfo, allocator::StandardCommandBufferAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::StandardMemoryAllocator,
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
    swapchain::{
        Surface, SurfaceApi, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
        SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};

#[derive(Resource)]
pub struct Renderer {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub queue_family_index: u32,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
}

fn create_instance() -> Result<Arc<Instance>, Box<dyn Error>> {
    let library = VulkanLibrary::new()?;
    let names = required_vulkan_extensions();
    if names.is_empty() {
        return Err("GLFW reports no Vulkan support (no required instance extensions)".into());
    }

    Ok(Instance::new(
        library,
        InstanceCreateInfo {
            enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_owned()],
            enabled_extensions: InstanceExtensions::from_iter(names.iter().map(String::as_str)),
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        },
    )?)
}

impl Renderer {
    fn new(instance: Arc<Instance>, surface: &Arc<Surface>) -> Result<Self, Box<dyn Error>> {
        let mut device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|p| {
                p.supported_extensions().contains(&device_extensions)
                    && (p.api_version() >= Version::V1_3
                        || p.supported_extensions().khr_dynamic_rendering)
            })
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .ok_or("No suitable Vulkan device found")?;

        if physical.api_version() < Version::V1_3 {
            device_extensions.khr_dynamic_rendering = true;
        }

        let (device, mut queues) = Device::new(
            physical,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                enabled_features: DeviceFeatures {
                    dynamic_rendering: true,
                    ..DeviceFeatures::empty()
                },
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().ok_or("Device returned no queue")?;

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        Ok(Self {
            instance,
            device,
            queue,
            queue_family_index,
            command_buffer_allocator,
            memory_allocator,
        })
    }
}

/// Pick the API from whichever surface extension GLFW made us enable,
/// instead of assuming Xlib.
fn surface_api(instance: &Instance) -> SurfaceApi {
    let e = instance.enabled_extensions();
    if e.khr_wayland_surface {
        SurfaceApi::Wayland
    } else if e.khr_xcb_surface {
        SurfaceApi::Xcb
    } else if e.khr_xlib_surface {
        SurfaceApi::Xlib
    } else if e.khr_win32_surface {
        SurfaceApi::Win32
    } else if e.ext_metal_surface {
        SurfaceApi::Metal
    } else {
        SurfaceApi::Xlib
    }
}

fn create_surface(
    instance: &Arc<Instance>,
    window: &glfw::PWindow,
) -> Result<Arc<Surface>, Box<dyn Error>> {
    let mut raw_surface: u64 = 0;

    let result = unsafe {
        window.create_window_surface(
            instance.handle().as_raw() as _,
            std::ptr::null(),
            &mut raw_surface as *mut u64 as _,
        )
    };
    if result as i32 != 0 {
        return Err(format!("glfwCreateWindowSurface failed: {result:?}").into());
    }

    // SAFETY: valid surface created from this instance. Owner is None, so the
    // surface does not keep the GLFW window alive; it must be dropped before the
    // instance and before the window is destroyed.
    Ok(Arc::new(unsafe {
        Surface::from_handle(
            instance.clone(),
            SurfaceKHR::from_raw(raw_surface),
            surface_api(instance),
            None,
        )
    }))
}

fn create_swapchain(
    device: &Arc<Device>,
    surface: &Arc<Surface>,
    extent: [u32; 2],
) -> Result<WindowSwapchain, Box<dyn Error>> {
    let physical = device.physical_device();
    let caps = physical.surface_capabilities(surface, Default::default())?;
    let (format, color_space) = physical.surface_formats(surface, Default::default())?[0];

    let mut min_image_count = caps.min_image_count.max(2);
    if let Some(max) = caps.max_image_count {
        min_image_count = min_image_count.min(max);
    }

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count,
            image_format: format,
            image_color_space: color_space,
            image_extent: extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: caps
                .supported_composite_alpha
                .into_iter()
                .next()
                .ok_or("No supported composite alpha")?,
            ..Default::default()
        },
    )?;

    let views = images
        .iter()
        .map(|i| ImageView::new_default(i.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WindowSwapchain {
        swapchain,
        images,
        views,
        previous_frame_end: None,
        frame: None,
    })
}

fn recreate(sc: &mut WindowSwapchain, extent: [u32; 2]) -> Result<(), Box<dyn Error>> {
    let (swapchain, images) = sc.swapchain.recreate(SwapchainCreateInfo {
        image_extent: extent,
        ..sc.swapchain.create_info()
    })?;
    sc.views = images
        .iter()
        .map(|i| ImageView::new_default(i.clone()))
        .collect::<Result<_, _>>()?;
    sc.swapchain = swapchain;
    sc.images = images;
    Ok(())
}

#[derive(Component)]
pub struct WindowSurface {
    surface: Arc<Surface>,
}

#[derive(Component)]
pub struct WindowSwapchain {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub views: Vec<Arc<ImageView>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    pub frame: Option<FrameImage>,
}

pub struct FrameImage {
    pub image_index: u32,
    pub acquire_future: SwapchainAcquireFuture,
}

type PrimaryBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

thread_local! {
    static FRAMES: RefCell<HashMap<usize, PrimaryBuilder>> = RefCell::new(HashMap::new());
}

pub fn with_frame<R>(window_id: usize, f: impl FnOnce(&mut PrimaryBuilder) -> R) -> Option<R> {
    FRAMES.with(|cell| cell.borrow_mut().get_mut(&window_id).map(f))
}

#[derive(Component)]
pub struct RecreateSwapchain(u32, u32);

#[allow(clippy::type_complexity)]
pub(crate) fn init_renderers(
    mut cmd: Commands,
    windows: Query<(Entity, &GlfwWindowId), (Without<WindowSurface>, Without<NoRenderer>)>,
    renderer: Option<Res<Renderer>>,
    _marker: NonSendMarker,
) {
    let mut instance = renderer.as_ref().map(|r| r.instance.clone());

    for (entity, id) in windows.iter() {
        let inst = match &instance {
            Some(i) => i.clone(),
            None => match create_instance() {
                Ok(i) => i,
                Err(e) => panic!("Vulkan instance creation failed: {e}"),
            },
        };

        let surface = match with_window(id.0, |_, win| create_surface(&inst, win)) {
            Some(Ok(s)) => s,
            Some(Err(e)) => {
                log::error!("Surface creation failed: {e}");
                continue;
            }
            None => continue,
        };

        if instance.is_none() {
            let r = Renderer::new(inst.clone(), &surface).expect("renderer init failed");
            cmd.insert_resource(r);
            instance = Some(inst);
        }

        cmd.insert_component(entity, WindowSurface { surface });
    }
}

pub(crate) fn init_swapchains(
    mut cmd: Commands,
    renderer: Option<Res<Renderer>>,
    windows: Query<(Entity, &GlfwWindowId, &WindowSurface), Without<WindowSwapchain>>,
    _marker: NonSendMarker,
) {
    let Some(renderer) = renderer else { return };

    for (entity, id, ws) in windows.iter() {
        let Some((w, h)) = framebuffer_size(id.0) else {
            continue;
        };
        if w == 0 || h == 0 {
            continue;
        }

        match create_swapchain(&renderer.device, &ws.surface, [w, h]) {
            Ok(sc) => cmd.insert_component(entity, sc),
            Err(e) => log::error!("Swapchain creation failed: {e}"),
        }
    }
}

pub(crate) fn recreate_swapchains(
    mut cmd: Commands,
    mut q: Query<(Entity, &mut WindowSwapchain, &RecreateSwapchain)>,
    _marker: NonSendMarker,
) {
    for (entity, sc, req) in q.iter_mut() {
        if req.0 == 0 || req.1 == 0 {
            continue;
        }

        if sc.frame.is_some() {
            continue;
        }

        let current = sc.swapchain.image_extent();
        if current != [req.0, req.1]
            && let Err(e) = recreate(sc, [req.0, req.1])
        {
            log::error!("Swapchain recreate failed: {e}");
            continue;
        }

        cmd.remove_component::<RecreateSwapchain>(entity);
    }
}

pub(crate) fn setup_rendering_events(mut cmd: Commands) {
    cmd.on(recreate_event);
}

pub(crate) fn recreate_event(event: &WindowResized, mut cmd: Commands) {
    cmd.insert_component(event.entity, RecreateSwapchain(event.width, event.height));
}

pub(crate) fn frame_begin(
    mut cmd: Commands,
    mut q: Query<(Entity, &GlfwWindowId, &mut WindowSwapchain), Without<RecreateSwapchain>>,
    renderer: Option<Res<Renderer>>,
    _marker: NonSendMarker,
) {
    let Some(renderer) = renderer else { return };

    for (entity, id, sc) in q.iter_mut() {
        if sc.frame.is_some() {
            continue;
        }

        if let Some(prev) = sc.previous_frame_end.as_mut() {
            prev.cleanup_finished();
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(sc.swapchain.clone(), Some(Duration::from_millis(100))) {
                Ok(r) => r,
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    let [w, h] = sc.swapchain.image_extent();
                    cmd.insert_component(entity, RecreateSwapchain(w, h));
                    continue;
                }
                Err(Validated::Error(VulkanError::Timeout)) => continue,
                Err(e) => {
                    log::error!("Acquire failed: {e}");
                    continue;
                }
            };

        if suboptimal {
            let [w, h] = sc.swapchain.image_extent();
            cmd.insert_component(entity, RecreateSwapchain(w, h));
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            renderer.command_buffer_allocator.clone(),
            renderer.queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_rendering(RenderingInfo {
                color_attachments: vec![Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    clear_value: Some([0.05, 0.05, 0.1, 1.0].into()),
                    ..RenderingAttachmentInfo::image_view(sc.views[image_index as usize].clone())
                })],
                ..Default::default()
            })
            .unwrap();

        FRAMES.with(|f| f.borrow_mut().insert(id.0, builder));

        sc.frame = Some(FrameImage {
            image_index,
            acquire_future,
        });
    }
}

pub(crate) fn frame_finish(
    mut cmd: Commands,
    mut q: Query<(Entity, &GlfwWindowId, &mut WindowSwapchain)>,
    renderer: Option<Res<Renderer>>,
    _marker: NonSendMarker,
) {
    let Some(renderer) = renderer else { return };

    for (entity, id, sc) in q.iter_mut() {
        let Some(frame) = sc.frame.take() else {
            continue;
        };
        let FrameImage {
            image_index,
            acquire_future,
        } = frame;

        let Some(mut builder) = FRAMES.with(|f| f.borrow_mut().remove(&id.0)) else {
            continue;
        };

        builder.end_rendering().unwrap();

        let command_buffer = builder.build().unwrap();

        let prev = sc
            .previous_frame_end
            .take()
            .unwrap_or_else(|| sync::now(renderer.device.clone()).boxed_send_sync());

        let result = prev
            .join(acquire_future)
            .then_execute(renderer.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                renderer.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(sc.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match result {
            Ok(future) => sc.previous_frame_end = Some(future.boxed_send_sync()),
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                let [w, h] = sc.swapchain.image_extent();
                cmd.insert_component(entity, RecreateSwapchain(w, h));
                sc.previous_frame_end = Some(sync::now(renderer.device.clone()).boxed_send_sync());
            }
            Err(e) => {
                log::error!("Frame submit failed: {e}");
                sc.previous_frame_end = Some(sync::now(renderer.device.clone()).boxed_send_sync());
            }
        }
    }
}
