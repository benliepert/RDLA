use egui::{ClippedPrimitive, Context, TexturesDelta, FullOutput};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use pixels::{wgpu, PixelsContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;
use strum::IntoEnumIterator;
use std::sync::{Arc, Mutex};
use core::ops::RangeInclusive;

use crate::config::GridType;
use crate::colors::{ColorName, Theme};
use crate::Dla;
use crate::dla::{DEFAULT_BACK_CLR, DEFAULT_PART_CLR, DEFAULT_THEME};

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    gui: Gui,
}

/// Store application state here
struct Gui {
    /// Only show the egui window when true.
    window_open: bool,
    about_open: bool,

    paused: bool,

    grid_type: GridType,

    particles: usize, // to support progress bar

    stuck_particles: usize,

    complete: bool, // has the simulation completed?

    width: usize,
    height: usize,

    to_file: String,
    from_file: String,

    particle_color: ColorName,
    background_color: ColorName, 

    spawn_radius: usize,
    enable_spawn_radius: bool,

    // We have r/w access to the grid directly, so that we can just tell it how to update stuff in response to
    // certain gui changes
    arc: Arc<Mutex<Dla>>,

    // user can change the grid size in the gui. This is separate from width/height since those are the actual current
    // backend dimensions that are used for other calculations. Maybe I could just make them the same, since it would be
    // wrong for only a frame and nobody would notice
    selected_width: u32,
    selected_height: u32,

    // is time coloring enabled?
    time_coloring: bool,

    theme: Theme,
}

// These are public to allow some debugging in the backend
pub const PAUSE_BUTTON_TEXT: &str = "Stop";
pub const UNPAUSE_BUTTON_TEXT: &str = "Start";

const RESET_BUTTON_TEXT: &str = "Reset";
const SAVE_BUTTON_TEXT: &str = "Save to file";
const FROM_BUTTON_TEXT: &str = "Load from file";

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        arc: Arc<Mutex<Dla>>,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new(arc);

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window) -> FullOutput {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx);
        });

        let ret_output = output.clone();

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);

        ret_output
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );
        
        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

impl Gui {
    /// Create a `Gui`.
    /// Pass in an arc mutex to the simulation grid. This is used to synchronize data access/display,
    /// as the data needs to be shared between the display and computation thread
    fn new(arc: Arc<Mutex<Dla>>) -> Self {
        Self {
            window_open: true,
            about_open: true,
            paused: true,
            grid_type: GridType::Center,
            stuck_particles: 1,
            particles: 10000,
            complete: false,
            width: 400,
            height: 400,
            to_file: "".to_string(),
            from_file: "".to_string(),
            particle_color: DEFAULT_PART_CLR,
            background_color: DEFAULT_BACK_CLR,
            spawn_radius: 0, // particles can spawn anywhere to start
            enable_spawn_radius: false,
            selected_width: 400,
            selected_height: 400,
            time_coloring: true,
            arc: arc,
            theme: DEFAULT_THEME,
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Tools", |ui| {
                    if ui.button("Control Panel").clicked() {
                        self.window_open = true;
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() {
                        self.about_open = true;
                        ui.close_menu();
                    }
                })
            });
        });


        egui::Window::new("About")
            .open(&mut self.about_open)
            .show(ctx, |ui| {
                ui.label(format!("Diffusion Limited Aggregation (dla) is the clustering of particles undergoing a random walk due to Brownian Motion.{}",
                    if cfg!(target_arch = "wasm32") {
                        " Everything you see is rendered as textured triangles. There is no DOM, HTML, JS, or CSS. Just Rust."
                    } else {""}
                ));
            });

        egui::Window::new("Control Panel")
            .open(&mut self.window_open)
            .show(ctx, |ui| {

                // Grab everythign we need from the backend 
                match self.arc.lock() {
                    Ok(guard) => {
                        self.paused = guard.paused();
                        self.stuck_particles = guard.stuck_particles();
                        self.complete = guard.complete();
                        self.grid_type = guard.grid_type();
                        (self.width, self.height) = guard.size();
                    },
                    Err(poisoned) => {
                        panic!("Poisoned lock! ({})", poisoned);
                    }
                };

                // COMBO BOX grid type ---------------------
                ui.add_enabled_ui(self.paused || self.complete,|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Grid Type:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{}", self.grid_type))
                            .show_ui(ui, |ui| {
                                // TODO: could just set paused in here to avoid keeping the simulation in sync
                                let cur_grid_type = self.grid_type;
                                // add a value for every grid type, thanks scrum :)
                                for grid_type in GridType::iter() {
                                    ui.selectable_value(&mut self.grid_type, grid_type, grid_type.to_string());
                                }
                                if cur_grid_type != self.grid_type {
                                    // selection changed
                                    self.arc.lock().unwrap().handle_grid_type_selected(self.grid_type);
                                }
                        });
                    });
                });
                ui.collapsing("Colors", |ui| {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut self.time_coloring, "Theme:   ")
                            .on_hover_text("Particle color is based on when it stuck")
                            .clicked() {
                            self.arc.lock().unwrap().handle_theme_changed(
                                if self.time_coloring {
                                    Some(self.theme)
                                } else {
                                    None
                                });    
                        }
                        ui.add_enabled_ui(self.time_coloring, |ui| {
                            // NOTE: the label for this combobox is a hack since the label must be unique, but I want these 3
                            // combo boxes to have an empty label. I should learn how to use from_id_source()
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{}", self.theme))
                                .show_ui(ui, |ui| {
                                    let cur_theme = self.theme;
                                    // add a value for every color, thanks scrum :)
                                    for theme in Theme::iter() {
                                        ui.selectable_value(&mut self.theme, theme, theme.to_string());
                                    }
                                    if cur_theme != self.theme {
                                        // selection changed
                                        self.arc.lock().unwrap().handle_theme_changed(Some(self.theme));
                                    }
                            });
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(!self.time_coloring, |ui| {
                            ui.label("Particle:        ");
                            // NOTE: the label for this combobox is a hack since the label must be unique, but I want these 3
                            // combo boxes to have an empty label. I should learn how to use from_id_source()
                            egui::ComboBox::from_label("  ")
                                .selected_text(format!("{}", self.particle_color))
                                .show_ui(ui, |ui| {
                                    let cur_part_color = self.particle_color;
                                    // add a value for every color, thanks scrum :)
                                    for color in ColorName::iter() {
                                        ui.selectable_value(&mut self.particle_color, color, color.to_string());
                                    }
                                    if cur_part_color != self.particle_color {
                                        // selection changed
                                        self.arc.lock().unwrap().handle_particle_color_changed(self.particle_color);
                                    }
                            });
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Background:");
                        // NOTE: the label for this combobox is a hack since the label must be unique, but I want these 3
                        // combo boxes to have an empty label. I should learn how to use from_id_source()
                        egui::ComboBox::from_label("   ")
                            .selected_text(format!("{}", self.background_color))
                            .show_ui(ui, |ui| {
                                let cur_back_color = self.background_color;
                                // add a value for every color, thanks scrum :)
                                for color in ColorName::iter() {
                                    ui.selectable_value(&mut self.background_color, color, color.to_string());
                                }
                                if cur_back_color != self.background_color {
                                    // selection changed
                                    self.arc.lock().unwrap().handle_background_color_changed(self.background_color);
                                }
                        });
                    });
                });

                // GRID SIZE SPIN BOX ------------
                ui.horizontal(|ui| {
                    ui.label("Grid Size:");
                    ui.add_enabled_ui(self.paused || self.complete, |ui| {
                        let min_grid_size = 400;
                        let max_grid_size = 1200;
                        ui.add(egui::DragValue::new(&mut self.selected_width)
                            .speed(50)
                            .clamp_range(RangeInclusive::new(min_grid_size, max_grid_size)));
                    });
                    ui.label("x");
                    ui.add_enabled_ui(self.paused || self.complete, |ui| {
                        let min_grid_size = 400;
                        let max_grid_size = 1200;
                        ui.add(egui::DragValue::new(&mut self.selected_height)
                            .speed(50)
                            .clamp_range(RangeInclusive::new(min_grid_size, max_grid_size)));
                    });

                    // enable apply button if we're not running and the selected size is different than that
                    // of the currently loaded grid
                    let enable_apply_button = self.selected_width != self.width as u32 
                                                    || self.selected_height != self.height as u32 
                                                    && (self.paused || self.complete);
                    ui.add_enabled_ui(enable_apply_button, |ui| {
                        if ui.button("Apply")
                                .on_hover_text("Apply the specified grid size. This will reset the grid.")
                                .clicked() {
                            self.arc.lock().unwrap().handle_reset(self.selected_width, self.selected_height);
                        }
                    });
                });
                // Num particles drag value ------
                // range should be dynamically clamped based on grid size
                // on the low end, clamp to stuck particles. this saves us handling making #particles smaller
                // than what's currently on the grid
                ui.horizontal(|ui| {
                    ui.label(format!("Particles: {} /", self.stuck_particles));
                    ui.add_enabled_ui(self.paused || self.complete, |ui| {
                        let start_particle_range = self.stuck_particles;
                        let end_particle_range = self.width as f32 * self.height as f32 * 0.90;
                        let old_particles = self.particles;
                        ui.add(egui::DragValue::new(&mut self.particles)
                            .speed(250)
                            .clamp_range(RangeInclusive::new(start_particle_range, end_particle_range as usize)));
                        if old_particles != self.particles {
                            // Max particles updated
                            self.arc.lock().unwrap().handle_particles_changed(self.particles);
                        }
                    });

                    let old_enable_spawn_radius = self.enable_spawn_radius;
                    ui.checkbox(&mut self.enable_spawn_radius, "Spawn Radius:")
                        .on_hover_text("Particles will spawn at least this far from the center");
                    ui.add_enabled_ui( self.enable_spawn_radius, |ui| {
                        let start_radius = 0;
                        let end_radius = std::cmp::min(self.width, self.height) / 2;
                        let old_spawn_radius = self.spawn_radius;
                        ui.add(egui::DragValue::new(&mut self.spawn_radius)
                            .speed(20)
                            .clamp_range(RangeInclusive::new(start_radius, end_radius)));
                        if old_spawn_radius != self.spawn_radius {
                            // spawn radius updated
                            self.arc.lock().unwrap().handle_spawn_radius_changed(Some(self.spawn_radius));
                        }
                    });
                    if old_enable_spawn_radius != self.enable_spawn_radius && !self.enable_spawn_radius{
                        // if it changed, and now it's off, tell the backend to not use a radius anymore
                        self.arc.lock().unwrap().handle_spawn_radius_changed(None);
                    }
                });

                ui.horizontal(|ui| {
                    // PAUSE/RESET BUTTON ------------------
                    ui.add_enabled_ui(!self.complete && self.stuck_particles < self.particles, |ui| {
                        let button_text = if self.paused {
                            UNPAUSE_BUTTON_TEXT
                        } else {
                            PAUSE_BUTTON_TEXT
                        };
                        if ui.button(button_text).clicked() {
                            self.paused = !self.paused;
                            self.arc.lock().unwrap().handle_pause_button_clicked(button_text);
                        }
                    });
                    if ui.button(RESET_BUTTON_TEXT).clicked() {
                        self.paused = true; // auto pause on reset
                        self.arc.lock().unwrap().handle_reset(self.selected_width, self.selected_height);
                    }

                    // import/export is not supported on web yet
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // READ FROM / WRITE TO FILE -------
                        ui.collapsing("Import/Export", |ui| {
                            ui.separator();
                            // write to file button/text edit
                            ui.add_enabled_ui(self.paused | self.complete, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.to_file));

                                if ui.button(SAVE_BUTTON_TEXT).clicked() {
                                    // do nothing
                                    self.arc.lock().unwrap().handle_save_button_clicked(&self.to_file);
                                }
                            });
                            ui.separator();
                            // load from file
                            ui.add_enabled_ui(self.paused | self.complete, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.from_file));

                                if ui.button(FROM_BUTTON_TEXT).clicked() {
                                    // The backend will update particles/stuck particles accordingly
                                    self.arc.lock().unwrap().handle_from_button_clicked(&self.from_file);
                                }
                            });
                            ui.separator();
                        });
                    }
                });

                // PROGRESS BAR ------------------
                let progress: f32 = self.stuck_particles as f32 / self.particles as f32;
                let progress_bar = egui::ProgressBar::new(progress)
                    .show_percentage();
                ui.add(progress_bar);
            });
    }
}
