use std::cmp::Ordering;

use crate::{Geom, GeomType};

use itertools::Itertools;
use nalgebra::{Quaternion, Vector3};

#[derive(Debug, Clone)]
pub struct Body {
    pub id: i32,
    pub name: String,
    pub parent_id: i32,
    pub geom_n: i32,
    pub geom_addr: i32,
    pub pos: Vector3<f64>,
    pub quat: Quaternion<f64>,
}

impl Body {
    pub fn geoms(&self, geoms: &[Geom]) -> Vec<Geom> {
        let mut body_geoms = Vec::new();
        for i in 0..self.geom_n {
            let geom = &geoms[(self.geom_addr + i) as usize];
            body_geoms.push(geom.clone());
        }
        body_geoms
    }

    /// Return the body to be rendered
    pub fn render_geom(&self, geoms: &[Geom]) -> Option<Geom> {
        let geom_query = geoms.iter().filter(|g| g.body_id == self.id);
        if geom_query.clone().count() == 1 {
            return Some(geom_query.clone().last().unwrap().clone());
        }

        let geoms = self.geoms(geoms);

        // This is questionable, but it seems to work
        let geom = geoms
            .iter()
            .filter(|g| g.geom_group < 3)
            .sorted_by(|g1, g2| {
                if g1.geom_type == GeomType::MESH && g2.geom_type != GeomType::MESH {
                    return Ordering::Less;
                }
                if g1.geom_type != GeomType::MESH && g2.geom_type == GeomType::MESH {
                    return Ordering::Greater;
                }

                if g1.geom_group < g2.geom_group {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .last()
            .cloned();

        geom
    }
}
