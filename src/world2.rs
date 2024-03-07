use crate::world_map::WorldSampler;
use eframe::glow;
use eframe::glow::HasContext;
use std::sync::Arc;

//

pub struct WorldGLSL<C: HasContext> {
    pub program: C::Program,
    pub vertex_array: C::VertexArray,
    texture: C::Texture,
    // we can't persist these because they are not Send
    // sul_world: C::UniformLocation,
    // sul_matrix: C::UniformLocation,
}

/// object that can use GLSL to paint the world map as an ERP that has been rotated by a matrix.
impl<C: HasContext> WorldGLSL<C> {
    pub(crate) fn new(gl: &Arc<C>) -> Self {
        /* let shader_version = ShaderVersion::get(gl);

        if !shader_version.is_new_shader_interface() {
            panic!(
                "Custom 3D painting hasn't been ported to {:?}",
                shader_version
            );
        }*/

        unsafe {
            let program = Self::compile_program(
                gl,
                // &shader_version,
                include_str!("vertex.glsl"),
                include_str!("fragment.glsl"),
            );

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            let tex = { Self::world_map_texture(gl).unwrap() };
            /*    let sul_world = gl.get_uniform_location(program, "world").unwrap();
                        let sul_matrix = gl.get_uniform_location(program, "rotation").unwrap();
            */
            Self {
                program,
                vertex_array,
                texture: tex,
                // sul_world,
                // sul_matrix,
            }
        }
    }

    unsafe fn world_map_texture(gl: &Arc<C>) -> Result<C::Texture, String> {
        let image = WorldSampler::raw_world_map();
        let tex: C::Texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            image.width as i32,
            image.height as i32,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            Some(image.rgb_pixels.as_slice()),
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        Ok(tex)
    }

    unsafe fn compile_program(
        gl: &Arc<C>,
        // shader_version: &ShaderVersion,
        vertex_source: &str,
        fragment_source: &str,
    ) -> C::Program {
        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (vertex_source, fragment_source);

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let shaders: Vec<_> = shader_sources
            .iter()
            .map(|(shader_type, shader_source)| {
                Self::compile_attach_shader(
                    gl,
                    // *shader_version,
                    program,
                    *shader_type,
                    shader_source,
                )
            })
            .collect();

        gl.link_program(program);
        assert!(
            gl.get_program_link_status(program),
            "{}",
            gl.get_program_info_log(program)
        );

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        program
    }

    unsafe fn compile_attach_shader(
        gl: &C,
        // shader_version: ShaderVersion,
        program: C::Program,
        shader_type: u32,
        shader_source: &str,
    ) -> C::Shader {
        let shader = gl.create_shader(shader_type).expect("Cannot create shader");
        gl.shader_source(
            shader,
            &format!(
                "{}\n{}",
                "", //shader_version.version_declaration(),
                shader_source
            ),
        );
        gl.compile_shader(shader);
        assert!(
            gl.get_shader_compile_status(shader),
            "Failed to compile custom_3d_glow {shader_type}: {}",
            gl.get_shader_info_log(shader)
        );

        gl.attach_shader(program, shader);
        shader
    }

    pub(crate) fn paint(&self, gl: &Arc<C>, rotation: &[f32]) {
        unsafe {
            let sul_world: C::UniformLocation =
                gl.get_uniform_location(self.program, "world").unwrap();
            let sul_matrix: C::UniformLocation =
                gl.get_uniform_location(self.program, "rotation").unwrap();
            //gl.viewport(100, 100, 400, 200);
            gl.use_program(Some(self.program));
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.uniform_1_i32(Some(&sul_world), 0);
            gl.uniform_matrix_3_f32_slice(Some(&sul_matrix), false, rotation);
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}

impl<C: HasContext> Drop for WorldGLSL<C> {
    fn drop(&mut self) {
        todo!()
    }
}
