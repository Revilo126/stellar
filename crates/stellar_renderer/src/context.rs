use std::sync::Arc;

use fent_ecs::{
    prelude::*,
    system::{
        access::{Access, AccessError},
        meta::SystemMeta,
    },
    world::{change_detection::Tick, unsafe_world_cell::UnsafeWorldCell},
};
use stellar_window::{PrimaryWindow, glfw::GlfwWindowId};
use vulkano::{
    buffer::Subbuffer,
    pipeline::{GraphicsPipeline, graphics::viewport::Viewport},
};

use crate::{
    renderer::{Renderer, WindowSwapchain, with_frame},
    vertex::Vertex,
};

pub struct RenderContext<'w> {
    renderer: Option<Res<'w, Renderer>>,
    windows: Query<'w, (Entity, &'w GlfwWindowId, &'w WindowSwapchain)>,
    primary: Query<'w, Entity, With<PrimaryWindow>>,
    current_window: Option<Entity>,
}

impl RenderContext<'_> {
    pub fn set_window(&mut self, entity: Entity) -> &mut Self {
        self.current_window = Some(entity);
        self
    }

    pub fn set_primary_window(&mut self) -> &mut Self {
        self.current_window = self.primary.iter().next();
        self
    }

    pub fn clear_window(&mut self) -> &mut Self {
        self.current_window = None;
        self
    }

    fn for_targets(&self, mut f: impl FnMut(usize, [u32; 2], &Renderer)) {
        let Some(renderer) = self.renderer.as_deref() else {
            return;
        };
        for (entity, id, sc) in self.windows.iter() {
            if self.current_window.is_some_and(|c| c != entity) {
                continue;
            }
            f(id.0, sc.swapchain.image_extent(), renderer);
        }
    }

    pub fn bind_pipeline(&mut self, pipeline: &Arc<GraphicsPipeline>) -> &mut Self {
        self.for_targets(|window_id, extent, _| {
            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [extent[0] as f32, extent[1] as f32],
                depth_range: 0.0..=1.0,
            };
            with_frame(window_id, |b| {
                b.bind_pipeline_graphics(pipeline.clone()).unwrap();
                b.set_viewport(0, [viewport].into_iter().collect()).unwrap();
            });
        });
        self
    }

    pub fn draw(&mut self, vertices: &Subbuffer<[Vertex]>) -> &mut Self {
        let count = vertices.len() as u32;
        self.for_targets(|window_id, _, _| {
            with_frame(window_id, |b| {
                b.bind_vertex_buffers(0, vertices.clone()).unwrap();
                // SAFETY: bind_pipeline ran for this window earlier in the same system.
                unsafe { b.draw(count, 1, 0, 0) }.unwrap();
            });
        });
        self
    }

    pub fn renderer(&self) -> Option<&Renderer> {
        self.renderer.as_deref()
    }
}

impl SystemParam for RenderContext<'_> {
    type State = (
        <Option<Res<'static, Renderer>> as SystemParam>::State,
        <Query<'static, (Entity, &'static GlfwWindowId, &'static WindowSwapchain)> as SystemParam>::State,
        <Query<'static, Entity, With<PrimaryWindow>> as SystemParam>::State,
        <NonSendMarker as SystemParam>::State,
    );
    type Item<'w> = RenderContext<'w>;

    fn init_state(world: &mut World, meta: &mut SystemMeta) -> Self::State {
        (
            <Option<Res<Renderer>> as SystemParam>::init_state(world, meta),
            <Query<(Entity, &GlfwWindowId, &WindowSwapchain)> as SystemParam>::init_state(
                world, meta,
            ),
            <Query<Entity, With<PrimaryWindow>> as SystemParam>::init_state(world, meta),
            <NonSendMarker as SystemParam>::init_state(world, meta),
        )
    }

    fn access(state: &Self::State, access: &mut Access) -> Result<(), AccessError> {
        <Option<Res<Renderer>> as SystemParam>::access(&state.0, access)?;
        <Query<(Entity, &GlfwWindowId, &WindowSwapchain)> as SystemParam>::access(
            &state.1, access,
        )?;
        <Query<Entity, With<PrimaryWindow>> as SystemParam>::access(&state.2, access)?;
        <NonSendMarker as SystemParam>::access(&state.3, access)
    }

    unsafe fn get_param<'w>(
        state: &'w mut Self::State,
        meta: &SystemMeta,
        world: UnsafeWorldCell<'w>,
        this_run: Tick,
    ) -> Self::Item<'w> {
        let (renderer_s, windows_s, primary_s, marker_s) = state;

        // SAFETY: access was already defined
        unsafe {
            let _ = <NonSendMarker as SystemParam>::get_param(marker_s, meta, world, this_run);

            RenderContext {
                renderer: <Option<Res<Renderer>> as SystemParam>::get_param(
                    renderer_s, meta, world, this_run,
                ),
                windows:
                    <Query<(Entity, &GlfwWindowId, &WindowSwapchain)> as SystemParam>::get_param(
                        windows_s, meta, world, this_run,
                    ),
                primary: <Query<Entity, With<PrimaryWindow>> as SystemParam>::get_param(
                    primary_s, meta, world, this_run,
                ),
                current_window: None,
            }
        }
    }
}
