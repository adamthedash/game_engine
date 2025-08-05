use std::sync::Arc;

use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    error::ExternalError,
    window::{CursorGrabMode, Window},
};

pub struct DrawContext {
    pub window: Arc<Window>,
    surface: Surface<'static>,
    pub config: SurfaceConfiguration,

    _gpu_handle: Instance,
    pub device: Device,
    pub queue: Queue,
}

impl DrawContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let (surface, gpu_handle, device, queue, config) = Self::init_gpu(window.clone()).await;

        Self {
            window,
            surface,
            config,
            _gpu_handle: gpu_handle,
            device,
            queue,
        }
    }

    async fn init_gpu<'a>(
        window: Arc<Window>,
    ) -> (Surface<'a>, Instance, Device, Queue, SurfaceConfiguration) {
        // Get handle to GPU
        let gpu_handle = Instance::new(&InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });

        // Create a texture surface - this is what we draw on
        let surface = gpu_handle.create_surface(window.clone()).unwrap();
        // Adapter is another kind of handle to GPU
        let adapter = gpu_handle
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Device - Yet another GPU handle
        // Queue - Used to send draw operations to the GPU
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // Configure the draw surface to match the window
        let window_size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &config);

        (surface, gpu_handle, device, queue, config)
    }

    // Resize the window
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.height = size.height;
        self.config.width = size.width;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn get_texture_view(&self) -> (SurfaceTexture, TextureView) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        (output, view)
    }

    /// Grab cursor control so the camera can be moved around
    pub fn grab_cursor(&self) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(CursorGrabMode::Confined)?;
        self.window.set_cursor_visible(false);

        // Centre the cursor in the window
        self.centre_cursor()
    }

    /// Unlock the cursor so the player can interact with UI
    pub fn ungrab_cursor(&self) {
        self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        self.window.set_cursor_visible(true);
    }

    /// Set the cursor's position, position is in pixel space
    pub fn set_cursor_pos(&self, position: &PhysicalPosition<u32>) -> Result<(), ExternalError> {
        // On Wayland we need to lock it first
        let wayland = self.window.set_cursor_grab(CursorGrabMode::Locked).is_ok();

        self.window.set_cursor_position(*position)?;

        if wayland {
            self.window.set_cursor_grab(CursorGrabMode::Confined)?;
        }
        Ok(())
    }

    /// Centre the cursor on the screen
    pub fn centre_cursor(&self) -> Result<(), ExternalError> {
        self.set_cursor_pos(&PhysicalPosition::new(
            self.config.width / 2,
            self.config.height / 2,
        ))
    }
}
