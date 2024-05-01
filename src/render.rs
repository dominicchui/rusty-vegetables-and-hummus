use gl::types::GLuint;
use nalgebra::{Matrix3, Matrix4, Vector2, Vector3};
use std::ffi::CString;

use crate::{
    camera::Camera,
    constants,
    ecology::{Bushes, Cell, CellIndex, Ecosystem, Trees},
};

pub(crate) struct EcosystemRenderable {
    pub(crate) ecosystem: Ecosystem,
    pub(crate) m_camera: Camera,
    m_vao: GLuint,
    m_lines_vao: GLuint,
    m_vbo: GLuint,
    m_lines_vbo: GLuint,
    m_ibo: GLuint,
    m_lines_ibo: GLuint,
    m_num_vertices: GLuint,          // verts.len()
    m_num_drawable_vertices: GLuint, // faces.len() * 3
    m_num_line_vertices: GLuint,
    m_model_matrix: Matrix4<f32>,
    m_vertices: Vec<Vector3<f32>>,
}

impl EcosystemRenderable {
    pub fn init() -> Self {
        let mut ecosystem = Ecosystem::init_standard();
        ecosystem.recompute_sunlight();

        // initialize based on the cell grid of the ecosystem
        let num_cells = constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH;
        let mut verts: Vec<Vector3<f32>> = vec![];
        let mut normals: Vec<Vector3<f32>> = vec![];
        let mut faces: Vec<Vector3<i32>> = vec![];
        let mut colors: Vec<Vector3<f32>> = vec![];
        let mut lines: Vec<Vector2<i32>> = vec![];
        verts.reserve(num_cells);
        normals.reserve(num_cells);

        for i in 0..constants::AREA_SIDE_LENGTH {
            for j in 0..constants::AREA_SIDE_LENGTH {
                let index = CellIndex::new(i, j);
                let cell = &ecosystem[index];
                let height = cell.get_height();
                verts.push(Vector3::new(i as f32, j as f32, height));
                normals.push(ecosystem.get_normal(index));
                colors.push(ecosystem.get_color(index));
            }
        }
        // simple tessellation of square grid
        for i in 0i32..constants::AREA_SIDE_LENGTH as i32 - 1 {
            for j in 0i32..constants::AREA_SIDE_LENGTH as i32 - 1 {
                // build two triangles
                let index = get_flat_index(i, j);
                let right = get_flat_index(i + 1, j);
                let bottom = get_flat_index(i, j + 1);
                let bottom_right = get_flat_index(i + 1, j + 1);
                faces.push(Vector3::new(index, bottom, right));
                faces.push(Vector3::new(bottom, bottom_right, right));

                lines.push(Vector2::new(index, right));
                lines.push(Vector2::new(index, bottom));
                lines.push(Vector2::new(right, bottom_right));
                lines.push(Vector2::new(bottom, bottom_right));
            }
        }

        // add trees and bushes
        for i in 0..constants::AREA_SIDE_LENGTH {
            for j in 0..constants::AREA_SIDE_LENGTH {
                let index = CellIndex::new(i, j);
                let cell = &ecosystem[index];
                let center: Vector3<f32> = Vector3::new(i as f32, j as f32, cell.get_height());
                Self::add_tree(center, cell.get_height_of_trees() / 10.0, &mut verts, &mut normals, &mut colors, &mut faces);
                Self::add_dead(center, cell.get_dead_vegetation() / 10.0, &mut verts, &mut normals, &mut colors, &mut faces);
                // Self::add_bush(center, cell.estimate_bush_biomass(), &mut verts, &mut normals, &mut colors, &mut faces);
            }
        }

        let mut ecosystem_render = EcosystemRenderable {
            ecosystem,
            m_vao: 0,
            m_vbo: 0,
            m_ibo: 0,
            m_num_vertices: 0,
            m_num_drawable_vertices: 0,
            m_model_matrix: Matrix4::identity(),
            m_vertices: vec![],
            m_camera: Camera::init(),
            m_lines_vao: 0,
            m_lines_vbo: 0,
            m_lines_ibo: 0,
            m_num_line_vertices: 0,
        };

        // Initialize camera in reasonable location
        let near_plane = 0.001;
        let far_plane = 10000.0;
        let middle = constants::AREA_SIDE_LENGTH as f32 / 2.0;
        let center = Vector3::new(middle, middle, constants::DEFAULT_BEDROCK_HEIGHT);
        let eye: Vector3<f32> = center + Vector3::new (0.0, 15.0, 15.0);
            // + Vector3::new(
            //     0.0,
            //     2.0 * constants::AREA_SIDE_LENGTH as f32,
            //     1.5 * constants::AREA_SIDE_LENGTH as f32,
            // );
        let target: Vector3<f32> = center;
        println!("center {center:?}");
        println!("eye {eye:?}");
        ecosystem_render.m_camera.look_at(eye, target);
        ecosystem_render.m_camera.set_orbit_point(target);
        ecosystem_render.m_camera.set_perspective(
            120.0,
            constants::SCREEN_WIDTH as f32 / constants::SCREEN_HEIGHT as f32,
            near_plane,
            far_plane,
        );

        unsafe {
            gl::GenBuffers(1, &mut ecosystem_render.m_vbo);
            gl::GenBuffers(1, &mut ecosystem_render.m_ibo);
            gl::GenVertexArrays(1, &mut ecosystem_render.m_vao);

            EcosystemRenderable::populate_vbo(ecosystem_render.m_vbo, &verts, &normals, &colors);
        }

        // set up IBO
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ecosystem_render.m_ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (std::mem::size_of::<i32>() * 3 * faces.len()) as gl::types::GLsizeiptr,
                faces.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            let mut err: gl::types::GLenum = gl::GetError();
            while err != gl::NO_ERROR {
                // Process/log the error.
                println!("ibo error {err}");
                err = gl::GetError();
            }
        }

        // set up VAO
        unsafe {
            gl::BindVertexArray(ecosystem_render.m_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, ecosystem_render.m_vbo);

            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0,                // index of the generic vertex attribute ("layout (location = 0)")
                3,                // the number of components per generic vertex attribute
                gl::FLOAT,        // data type
                gl::FALSE,        // normalized (int-to-float conversion)
                0,                // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                (std::mem::size_of::<f32>() * verts.len() * 3) as *const gl::types::GLvoid,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                (std::mem::size_of::<f32>() * (verts.len() * 3 + colors.len() * 3))
                    as *const gl::types::GLvoid,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ecosystem_render.m_ibo);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            let mut err: gl::types::GLenum = gl::GetError();
            while err != gl::NO_ERROR {
                // Process/log the error.
                println!("vao error {err}");
                err = gl::GetError();
            }
        }

        // set up VAO, VBO, and IBO for lines
        unsafe {
            gl::GenBuffers(1, &mut ecosystem_render.m_lines_vbo);
            gl::GenBuffers(1, &mut ecosystem_render.m_lines_ibo);
            gl::GenVertexArrays(1, &mut ecosystem_render.m_lines_vao);

            // VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, ecosystem_render.m_lines_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * (verts.len() * 3)) as gl::types::GLsizeiptr,
                verts.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            // IBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ecosystem_render.m_lines_ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (std::mem::size_of::<i32>() * 2 * lines.len()) as gl::types::GLsizeiptr,
                lines.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            // VAO
            gl::BindVertexArray(ecosystem_render.m_lines_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, ecosystem_render.m_lines_vbo);

            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0,                // index of the generic vertex attribute ("layout (location = 0)")
                3,                // the number of components per generic vertex attribute
                gl::FLOAT,        // data type
                gl::FALSE,        // normalized (int-to-float conversion)
                0,                // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ecosystem_render.m_lines_ibo);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        ecosystem_render.m_vertices = verts;
        ecosystem_render.m_num_vertices = num_cells as u32;
        ecosystem_render.m_num_drawable_vertices = faces.len() as u32 * 3;
        ecosystem_render.m_num_line_vertices = lines.len() as u32 * 2;

        ecosystem_render
    }

    fn add_tree(
        center: Vector3<f32>,
        height: f32,
        verts: &mut Vec<Vector3<f32>>,
        normals: &mut Vec<Vector3<f32>>,
        colors: &mut Vec<Vector3<f32>>,
        faces: &mut Vec<Vector3<i32>>
    ) {
        let diameter = Trees::estimate_diameter_from_height(height);
        let resolution: i32 = 16; // Number of sides in the cylinder

        // Calculate vertices and normals for the cylinder
        let mut cylinder_verts: Vec<Vector3<f32>> = vec![];
        let mut cylinder_normals: Vec<Vector3<f32>> = Vec::new();
        for i in 0..resolution {
            let phi: f32 = 4.0 * std::f32::consts::PI * (i as f32) / (resolution as f32);
            let x: f32 = center.x + diameter * 0.5 * phi.cos();
            let y: f32 = center.y + diameter * 0.5 * phi.sin();
            let z: f32 = center.z;
            cylinder_verts.push(Vector3::new(x, y, z));
            cylinder_verts.push(Vector3::new(x, y, z + height));
            cylinder_normals.push(Vector3::new(-phi.cos(), 0.0,  -phi.sin()));
            cylinder_normals.push(Vector3::new(-phi.cos(), 0.0,  -phi.sin()));
        }

        // Add vertices, normals, and colors to the existing vectors
        let start_index: i32 = verts.len() as i32;
        verts.extend_from_slice(&cylinder_verts);
        normals.extend_from_slice(&cylinder_normals);
        colors.extend_from_slice(&vec![constants::TREES_COLOR; (resolution * 2) as usize]);

        // Add faces to connect the vertices
        for i in 0..resolution {
            let a = start_index + i;
            let b = start_index + (i + 1) % resolution;
            let c = start_index + (i + 2) % resolution;
            let d = start_index + (i + 3) % resolution;
            faces.push(Vector3::new(a, b, c));
            faces.push(Vector3::new(b, c, d));
        }
    }

    fn add_dead(
        center: Vector3<f32>,
        height: f32,
        verts: &mut Vec<Vector3<f32>>,
        normals: &mut Vec<Vector3<f32>>,
        colors: &mut Vec<Vector3<f32>>,
        faces: &mut Vec<Vector3<i32>>
    ) {
        let diameter = Trees::estimate_diameter_from_height(height);
        let resolution: i32 = 16; // Number of sides in the cylinder

        // Calculate vertices and normals for the cylinder
        let mut cylinder_verts: Vec<Vector3<f32>> = vec![];
        let mut cylinder_normals: Vec<Vector3<f32>> = Vec::new();
        for i in 0..resolution {
            let phi: f32 = 4.0 * std::f32::consts::PI * (i as f32) / (resolution as f32);
            let x = center.x - 0.5;
            let y = center.y + diameter * 0.5 * phi.cos();
            let z = center.z + diameter * 0.5 * (1.0 + phi.sin());
            cylinder_verts.push(Vector3::new(x, y, z));
            cylinder_verts.push(Vector3::new(x + height, y, z));
            cylinder_normals.push(Vector3::new(phi.cos(), 0.0, phi.sin()));
            cylinder_normals.push(Vector3::new(phi.cos(), 0.0, phi.sin()));
        }

        // Add vertices, normals, and colors to the existing vectors
        let start_index: i32 = verts.len() as i32;
        verts.extend_from_slice(&cylinder_verts);
        normals.extend_from_slice(&cylinder_normals);
        colors.extend_from_slice(&vec![constants::DEAD_COLOR; (resolution * 2) as usize]);

        // Add faces to connect the vertices
        for i in 0..resolution {
            let a = start_index + i;
            let b = start_index + (i + 1) % resolution;
            let c = start_index + (i + 2) % resolution;
            let d = start_index + (i + 3) % resolution;
            faces.push(Vector3::new(a, b, c));
            faces.push(Vector3::new(b, c, d));
        }
    }

    fn add_bush(
        center: Vector3<f32>,
        biomass: f32,
        verts: &mut Vec<Vector3<f32>>,
        normals: &mut Vec<Vector3<f32>>,
        colors: &mut Vec<Vector3<f32>>,
        faces: &mut Vec<Vector3<i32>>
    ) {
        let diameter = Bushes::estimate_crown_area_from_biomass(biomass);
        let resolution: i32 = 16;

        let mut hsphere_verts: Vec<Vector3<f32>> = vec![];
        let mut hsphere_normals: Vec<Vector3<f32>> = Vec::new();
        for i in 0..resolution {
            let phi: f32 = 4.0 * std::f32::consts::PI * (i as f32) / (resolution as f32);
            for j in 0..resolution {
                let theta = 2.0 * std::f32::consts::PI * (j as f32) / (resolution as f32);
                let x = center.x + diameter * 0.5 * phi.sin() * theta.cos();
                let y = center.y + diameter * 0.5 * phi.sin() * theta.sin();
                let z = center.z + diameter * 0.5 * phi.cos();
                hsphere_verts.push(Vector3::new(x, y, z));
                hsphere_normals.push(Vector3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos()));
            }
        }

        // Add vertices, normals, and colors to the existing vectors
        let start_index: i32 = verts.len() as i32;
        verts.extend_from_slice(&hsphere_verts);
        normals.extend_from_slice(&hsphere_normals);
        colors.extend_from_slice(&vec![constants::BUSHES_COLOR; hsphere_verts.len()]);

        // Add faces to connect the vertices
        for i in 0..resolution {
            let a = start_index + i;
            let b = start_index + (i + 1) % resolution;
            let c = start_index + (i + 2) % resolution;
            let d = start_index + (i + 3) % resolution;
            faces.push(Vector3::new(a, b, c));
            faces.push(Vector3::new(b, c, d));
        }
    }

    fn populate_vbo(
        m_vbo: GLuint,
        verts: &[Vector3<f32>],
        normals: &[Vector3<f32>],
        colors: &[Vector3<f32>],
    ) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, m_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>()
                    * ((verts.len() * 3) + (normals.len() * 3) + (colors.len() * 3)))
                    as gl::types::GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (std::mem::size_of::<f32>() * verts.len() * 3) as gl::types::GLsizeiptr,
                verts.as_ptr() as *const gl::types::GLvoid,
            );
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * verts.len() * 3) as gl::types::GLsizeiptr,
                (std::mem::size_of::<f32>() * normals.len() * 3) as gl::types::GLsizeiptr,
                normals.as_ptr() as *const gl::types::GLvoid,
            );
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * ((verts.len() * 3) + (normals.len() * 3)))
                    as gl::types::GLsizeiptr,
                (std::mem::size_of::<f32>() * colors.len() * 3) as gl::types::GLsizeiptr,
                colors.as_ptr() as *const gl::types::GLvoid,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            let mut err: gl::types::GLenum = gl::GetError();
            while err != gl::NO_ERROR {
                // Process/log the error.
                println!("vbo error {err}");
                err = gl::GetError();
            }
        }
    }

    pub fn update_vertices(&mut self) {
        let mut verts: Vec<Vector3<f32>> = vec![];
        let mut normals: Vec<Vector3<f32>> = vec![];
        let mut colors: Vec<Vector3<f32>> = vec![];
        let mut faces: Vec<Vector3<i32>> = vec![];
        for i in 0..constants::AREA_SIDE_LENGTH {
            for j in 0..constants::AREA_SIDE_LENGTH {
                let index = CellIndex::new(i, j);
                let cell = &self.ecosystem[index];
                // make uniform cube cells
                let height = cell.get_height();
                verts.push(Vector3::new(i as f32, j as f32, height));
                normals.push(self.ecosystem.get_normal(index));
                colors.push(self.ecosystem.get_color(index));
            }
        }

        // add trees and bushes
        for i in 0..constants::AREA_SIDE_LENGTH {
            for j in 0..constants::AREA_SIDE_LENGTH {
                let index = CellIndex::new(i, j);
                let cell = &self.ecosystem[index];
                let center: Vector3<f32> = Vector3::new(i as f32, j as f32, cell.get_height());
                Self::add_tree(center, cell.get_height_of_trees() / 10.0, &mut verts, &mut normals, &mut colors, &mut faces);
                Self::add_dead(center, cell.get_dead_vegetation() / 10.0, &mut verts, &mut normals, &mut colors, &mut faces);
                // Self::add_bush(center, cell.estimate_bush_biomass(), &mut verts, &mut normals, &mut colors, &mut faces);
            }
        }

        EcosystemRenderable::populate_vbo(self.m_vbo, &verts, &normals, &colors);
    }

    pub fn draw(&mut self, program_id: GLuint, render_mode: gl::types::GLuint) {
        if render_mode == gl::LINES {
            unsafe {
                let c_str = CString::new("wire").unwrap();
                let wire_loc = gl::GetUniformLocation(program_id, c_str.as_ptr());
                assert!(wire_loc != -1);
                gl::Uniform1i(wire_loc, 1);
            }
        } else {
            unsafe {
                let c_str = CString::new("wire").unwrap();
                let wire_loc = gl::GetUniformLocation(program_id, c_str.as_ptr());
                assert!(wire_loc != -1);
                gl::Uniform1i(wire_loc, 0);
            }
        }
        // set view and proj matrices
        unsafe {
            let c_str = CString::new("view").unwrap();
            let view = self.m_camera.get_view();
            let view_loc = gl::GetUniformLocation(program_id, c_str.as_ptr());
            assert!(view_loc != -1);
            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, &view[0]);

            let c_str = CString::new("proj").unwrap();
            let proj = self.m_camera.get_projection();
            let proj_loc = gl::GetUniformLocation(program_id, c_str.as_ptr());
            assert!(proj_loc != -1);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, &proj[0]);
        }

        let m3: Matrix3<f32> = Matrix3::new(
            self.m_model_matrix[0],
            self.m_model_matrix[1],
            self.m_model_matrix[2],
            self.m_model_matrix[4],
            self.m_model_matrix[5],
            self.m_model_matrix[6],
            self.m_model_matrix[8],
            self.m_model_matrix[9],
            self.m_model_matrix[10],
        );
        let inverse_transpose_model = m3.try_inverse().unwrap().transpose();
        unsafe {
            let c_str = CString::new("model").unwrap();
            let model_loc = gl::GetUniformLocation(program_id, c_str.as_ptr());
            assert!(model_loc != -1);
            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, &self.m_model_matrix[0]);
            let c_str = CString::new("inverseTransposeModel").unwrap();
            let inv_model_loc = gl::GetUniformLocation(program_id, c_str.as_ptr());
            assert!(inv_model_loc != -1);
            gl::UniformMatrix3fv(inv_model_loc, 1, gl::FALSE, &inverse_transpose_model[0]);

            gl::BindVertexArray(self.m_vao);
            gl::Enable(gl::LINE_SMOOTH);
            gl::DrawElements(
                render_mode,
                self.m_num_drawable_vertices as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            let mut err: gl::types::GLenum = gl::GetError();
            while err != gl::NO_ERROR {
                // Process/log the error.
                println!("draw error {err}");
                err = gl::GetError();
            }

            gl::BindVertexArray(0);
        }
    }
}

// converts (x,y) index in 2D vec into an index into a flattened 1D vec
fn get_flat_index(x: i32, y: i32) -> i32 {
    y * constants::AREA_SIDE_LENGTH as i32 + x
}
