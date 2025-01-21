// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "PBR!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(-3.0, 1.0, 2.5),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);
    let mut gui = three_d::GUI::new(&context);

    let mut loaded = if let Ok(loaded) = three_d_asset::io::load_async(&[
        "../assets/chinese_garden_4k.hdr", // Source: https://polyhaven.com/
        "examples/assets/gltf/DamagedHelmet.glb", // Source: https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0
    ])
    .await
    {
        loaded
    } else {
        three_d_asset::io::load_async(&[
            "https://asny.github.io/three-d/assets/chinese_garden_4k.hdr",
            "examples/assets/gltf/DamagedHelmet.glb",
        ])
        .await
        .expect("failed to download the necessary assets, to enable running this example offline, place the relevant assets in a folder called 'assets' next to the three-d source")
    };

    let environment_map = loaded.deserialize("chinese").unwrap();
    let skybox = Skybox::new_from_equirectangular(&context, &environment_map);

    let mut cpu_model: CpuModel = loaded.deserialize("DamagedHelmet").unwrap();
    cpu_model
        .geometries
        .iter_mut()
        .for_each(|m| m.compute_tangents());
    let mut model = Model::<PhysicalMaterial>::new(&context, &cpu_model)
        .unwrap()
        .remove(0);

    let mut sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(16)),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    #[derive(Copy, Clone, PartialEq, Eq)]
    enum Orientation {
        XPos,
        XNeg,
        YPos,
        YNeg,
        ZPos,
        ZNeg,
    }

    fn orientation_to_vec3(orientation: Orientation) -> Vec3 {
        match orientation {
            Orientation::XPos => vec3(1.0, 0.0, 0.0),
            Orientation::XNeg => vec3(-1.0, 0.0, 0.0),
            Orientation::YPos => vec3(0.0, 1.0, 0.0),
            Orientation::YNeg => vec3(0.0, -1.0, 0.0),
            Orientation::ZPos => vec3(0.0, 0.0, 1.0),
            Orientation::ZNeg => vec3(0.0, 0.0, -1.0),
        }
    }

    fn get_orientation_coord(orientation: Orientation, coord: &mut Vec3) -> &mut f32 {
        match orientation {
            Orientation::XPos => &mut coord.x,
            Orientation::XNeg => &mut coord.x,
            Orientation::YPos => &mut coord.y,
            Orientation::YNeg => &mut coord.y,
            Orientation::ZPos => &mut coord.z,
            Orientation::ZNeg => &mut coord.z,
        }
    }

    fn orientation_to_string(orientation: Orientation) -> &'static str {
        match orientation {
            Orientation::XPos => "X+",
            Orientation::XNeg => "X-",
            Orientation::YPos => "Y+",
            Orientation::YNeg => "Y-",
            Orientation::ZPos => "Z+",
            Orientation::ZNeg => "Z-",
        }
    }

    const ORIENTATIONS: [Orientation; 6] = [
        Orientation::XPos,
        Orientation::XNeg,
        Orientation::YPos,
        Orientation::YNeg,
        Orientation::ZPos,
        Orientation::ZNeg,
    ];

    let mut clip_plane_enabled = false;
    let mut clip_plane_pos = vec3(0.0, 0.0, 0.0);
    let mut clip_plane_orientation = Orientation::ZPos;

    let light = AmbientLight::new_with_environment(&context, 1.0, Srgba::WHITE, skybox.texture());

    // main loop
    let mut normal_map_enabled = true;
    let mut occlusion_map_enabled = true;
    let mut metallic_roughness_enabled = true;
    let mut albedo_map_enabled = true;
    let mut emissive_map_enabled = true;
    window.render_loop(move |mut frame_input| {
        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");
                    ui.checkbox(&mut albedo_map_enabled, "Albedo map");
                    ui.checkbox(&mut metallic_roughness_enabled, "Metallic roughness map");
                    ui.checkbox(&mut normal_map_enabled, "Normal map");
                    ui.checkbox(&mut occlusion_map_enabled, "Occlusion map");
                    ui.checkbox(&mut emissive_map_enabled, "Emissive map");
                    ui.separator();
                    ui.checkbox(&mut clip_plane_enabled, "Clip plane");
                    if clip_plane_enabled {
                        ui.add(
                            Slider::new(
                                get_orientation_coord(clip_plane_orientation, &mut clip_plane_pos),
                                -1.0..=1.0,
                            )
                            .text("Clip plane offset"),
                        );
                        ComboBox::from_label("Clip plane orientation")
                            .selected_text(orientation_to_string(clip_plane_orientation))
                            .show_ui(ui, |ui| {
                                for orientation in ORIENTATIONS.iter() {
                                    ui.selectable_value(
                                        &mut clip_plane_orientation,
                                        *orientation,
                                        orientation_to_string(*orientation),
                                    );
                                }
                            });
                    }
                });
                panel_width = gui_context.used_rect().width();
            },
        );

        let clip_plane = if clip_plane_enabled {
            Some(ClipPlane::new(
                clip_plane_pos,
                orientation_to_vec3(clip_plane_orientation),
            ))
        } else {
            None
        };
        model.geometry.set_clip_plane(clip_plane);
        sphere.set_transformation(Mat4::from_translation(clip_plane_pos) * Mat4::from_scale(0.05));

        let viewport = Viewport {
            x: (panel_width * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width
                - (panel_width * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };
        camera.set_viewport(viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.5, 0.5, 0.5, 1.0, 1.0))
            .render(&camera, &skybox, &[])
            .write(|| {
                let material = PhysicalMaterial {
                    name: model.material.name.clone(),
                    albedo: model.material.albedo,
                    albedo_texture: if albedo_map_enabled {
                        model.material.albedo_texture.clone()
                    } else {
                        None
                    },
                    metallic: model.material.metallic,
                    roughness: model.material.roughness,
                    metallic_roughness_texture: if metallic_roughness_enabled {
                        model.material.metallic_roughness_texture.clone()
                    } else {
                        None
                    },
                    normal_scale: model.material.normal_scale,
                    normal_texture: if normal_map_enabled {
                        model.material.normal_texture.clone()
                    } else {
                        None
                    },
                    occlusion_strength: model.material.occlusion_strength,
                    occlusion_texture: if occlusion_map_enabled {
                        model.material.occlusion_texture.clone()
                    } else {
                        None
                    },
                    emissive: if emissive_map_enabled {
                        model.material.emissive
                    } else {
                        Srgba::BLACK
                    },
                    emissive_texture: if emissive_map_enabled {
                        model.material.emissive_texture.clone()
                    } else {
                        None
                    },
                    render_states: model.material.render_states,
                    is_transparent: model.material.is_transparent,
                    lighting_model: LightingModel::Cook(
                        NormalDistributionFunction::TrowbridgeReitzGGX,
                        GeometryFunction::SmithSchlickGGX,
                    ),
                };
                model.render_with_material(&material, &camera, &[&light]);
                gui.render()
            })
            .unwrap();

        FrameOutput::default()
    });
}
