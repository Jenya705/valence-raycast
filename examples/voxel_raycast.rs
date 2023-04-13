use bevy_ecs::prelude::*;
use valence::{client::movement::Movement, prelude::*};
use valence_raycast::voxel_raycast;

#[derive(Resource, Debug)]
struct RayToggle(pub bool);

#[derive(Component, Debug, Default)]
struct Ray {
    pub pos: DVec3,
    pub dir: DVec3,
}

fn main() {
    App::new()
        .add_plugin(ServerPlugin::new(()))
        .add_startup_system(setup)
        .add_systems((toggle_ray, init_player, draw_ray))
        .insert_resource(RayToggle(true))
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Query<&DimensionType>,
    biomes: Query<&Biome>,
) {
    let mut instance = Instance::new(
        Ident::new("overworld").unwrap(),
        &dimensions,
        &biomes,
        &server,
    );

    for z in -5..5 {
        for x in -5..5 {
            instance.insert_chunk([x, z], Chunk::default());
        }
    }

    for z in -100..100 {
        for x in -100..100 {
            instance.set_block([x, 64, z], BlockState::DIRT);
        }
    }

    commands.spawn(instance);
}

fn init_player(
    mut commands: Commands,
    mut new_client_q: Query<(&mut Location, &mut Position, Entity), Added<Client>>,
    instance_q: Query<Entity, With<Instance>>,
) {
    for (mut loc, mut pos, client) in new_client_q.iter_mut() {
        loc.0 = instance_q.single();
        pos.0 = DVec3::new(0.0, 66.0, 0.0);
        commands.entity(client).insert(Ray::default());
    }
}

fn toggle_ray(
    mut sneaking_e: EventReader<Sneaking>,
    mut ray_toggle: ResMut<RayToggle>,
    client_q: Query<(&Location, &Ray)>,
    mut instance_q: Query<&mut Instance>,
) {
    for event in sneaking_e.iter() {
        if SneakState::Start == event.state {
            ray_toggle.0 = !ray_toggle.0;
            if ray_toggle.0 {
                let (loc, ray) = client_q.get(event.client).unwrap();
                let mut instance = instance_q.get_mut(loc.0).unwrap();
                fill_ray(&mut instance, ray, BlockState::AIR);
            }
        }
    }
}

fn draw_ray(
    ray_toggle: Res<RayToggle>,
    mut movement_e: EventReader<Movement>,
    mut client_q: Query<(&Location, &mut Ray)>,
    mut instance_q: Query<&mut Instance>,
) {
    if ray_toggle.0 {
        for event in movement_e.iter() {
            let (loc, mut ray) = client_q.get_mut(event.client).unwrap();
            let mut instance = instance_q.get_mut(loc.0).unwrap();
            if ray.dir != DVec3::ZERO {
                fill_ray(&mut instance, &ray, BlockState::AIR);
            }
            let dir = event.look.vec().as_dvec3().normalize();
            ray.pos = event.position;
            ray.pos.y += 1.8;
            ray.pos += dir * 2.0;
            ray.dir = dir;
            fill_ray(&mut instance, &ray, BlockState::LIME_WOOL);
        }
    } else {
        movement_e.clear();
    }
}

fn fill_ray(instance: &mut Instance, ray: &Ray, block_state: BlockState) {
    voxel_raycast(
        ray.pos,
        ray.dir,
        50.0,
        |block_i, _, _| {
            if block_i.y > 64 {
                instance.set_block([block_i.x, block_i.y, block_i.z], block_state);
                false
            } else {
                true
            }
        },
    )
}
