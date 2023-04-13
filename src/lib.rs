use glam::*;
use parry3d_f64::{
    bounding_volume::Aabb,
    query::{Ray, RayCast},
};
use valence::prelude::*;
use voxel_tile_raycast::na::*;

pub fn voxel_raycast(
    origin: DVec3,
    dir: DVec3,
    max_dist: f64,
    mut func: impl FnMut(IVec3, DVec3, DVec3) -> bool,
) {
    voxel_tile_raycast::voxel_raycast(
        d2na(origin),
        d2na(dir),
        max_dist,
        |index, hit_pos, hit_normal| {
            let index = na2i(index);
            let hit_pos = na2d(hit_pos);
            let hit_normal = na2i(hit_normal);
            func(index, hit_pos, hit_normal.as_dvec3())
        },
    )
}

macro_rules! ray_cast {
    (
        $self: ident,
        $origin: ident,
        $dir: ident,
        $max_dist: ident,
        $func: ident,
        $get_block: ident
    ) => {{
        let na_origin = d2na($origin);
        let na_dir = d2na($dir).normalize();
        voxel_tile_raycast::voxel_raycast(na_origin, na_dir, $max_dist, move |index, _, _| {
            let index = na2i(index);
            let mut hit_pos = DVec3::ZERO;
            let mut hit_normal = DVec3::ZERO;
            let block = $self.$get_block([index.x, index.y, index.z]);
            if let Some(ref block) = block {
                for collision_shape in block.state().collision_shapes() {
                    let aabb = Aabb::new(
                        Point3::new(collision_shape[0], collision_shape[1], collision_shape[2]),
                        Point3::new(collision_shape[3], collision_shape[4], collision_shape[5]),
                    );

                    let bp = index.as_dvec3();

                    let ray = Ray::new(na_origin.into(), na_dir);

                    if let Some(ri) = aabb.cast_ray_and_get_normal(
                        &Isometry3::translation(bp.x, bp.y, bp.z),
                        &ray,
                        $max_dist,
                        true,
                    ) {
                        hit_normal = na2d(ri.normal);
                        hit_pos = na2d(ray.point_at(ri.toi).coords);
                    };
                }
            }
            if hit_normal != DVec3::ZERO {
                $func($self, index, hit_pos, hit_normal)
            } else {
                false
            }
        })
    }};
}

pub trait RayCastInstance {
    fn ray_cast_blocks(
        &self,
        origin: DVec3,
        dir: DVec3,
        max_dist: f64,
        func: impl FnMut(&Self, IVec3, DVec3, DVec3) -> bool,
    );

    fn ray_cast_mut_blocks(
        &mut self,
        origin: DVec3,
        dir: DVec3,
        max_dist: f64,
        func: impl FnMut(&mut Self, IVec3, DVec3, DVec3) -> bool,
    );
}

fn d2na(v: DVec3) -> Vector3<f64> {
    Vector3::new(v.x, v.y, v.z)
}

fn na2d(v: Vector3<f64>) -> DVec3 {
    DVec3::new(v.x, v.y, v.z)
}

fn na2i(v: Vector3<i32>) -> IVec3 {
    IVec3::new(v.x, v.y, v.z)
}

impl RayCastInstance for Instance {
    fn ray_cast_blocks(
        &self,
        origin: DVec3,
        dir: DVec3,
        max_dist: f64,
        mut func: impl FnMut(&Self, IVec3, DVec3, DVec3) -> bool,
    ) {
        ray_cast!(self, origin, dir, max_dist, func, block);
    }

    fn ray_cast_mut_blocks(
        &mut self,
        origin: DVec3,
        dir: DVec3,
        max_dist: f64,
        mut func: impl FnMut(&mut Self, IVec3, DVec3, DVec3) -> bool,
    ) {
        ray_cast!(self, origin, dir, max_dist, func, block_mut);
    }
}
