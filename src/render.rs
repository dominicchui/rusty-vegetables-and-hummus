use gl::types::GLuint;
use nalgebra::{Matrix4, Vector3};

use crate::{constants, ecology::Ecosystem};

struct EcosystemRenderable {
    ecosystem: Ecosystem,
    m_vao: GLuint,
    m_vbo: GLuint,
    m_ibo: GLuint,
    m_num_vertices: GLuint,
    m_model_matrix: Matrix4<f32>,
    m_vertices: Vec<Vector3<f32>>,
}

impl EcosystemRenderable {
    pub fn init(ecosystem: Ecosystem) -> Self {
        let ecosystem = EcosystemRenderable {
            ecosystem,
            m_vao: 0,
            m_vbo: 0,
            m_ibo: 0,
            m_num_vertices: 0,
            m_model_matrix: Matrix4::identity(),
            m_vertices: vec![],
        };

        // initialize based on the ecosystem
        let num_cells = constants::AREA_SIDE_LENGTH * constants::AREA_SIDE_LENGTH;
        let mut verts: Vec<Vector3<f32>> = vec![];
        let mut normals: Vec<Vector3<f32>> = vec![];
        let mut faces: Vec<Vector3<i32>> = vec![];
        verts.reserve(num_cells);
        normals.reserve(num_cells);
        // for (f, o) in triangles {
        //     let v1 = vertices[f[0] as usize];
        //     let v2 = vertices[f[1] as usize];
        //     let v3 = vertices[f[2] as usize];
        //     let e1 = v2 - v1;
        //     let e2 = v3 - v1;
        //     let mut n = e1.cross(&e2);
        //     if let Some(other_v) = o {
        //         let v = vertices[*other_v as usize];
        //         let pv = v - v1;
        //         if n.dot(&pv) > 0.0 {
        //             n = -n;
        //         }
        //     }
        //     let s: i32 = verts.len() as i32;
        //     faces.push(Vector3::new(s, s + 1, s + 2));
        //     normals.push(n);
        //     normals.push(n);
        //     normals.push(n);
        //     verts.push(v1);
        //     verts.push(v2);
        //     verts.push(v3);
        // }

        ecosystem
    }

    pub fn draw(&self) {
        
    }
}