use bevy::prelude::*;
use mapmap_core::module::BevyShapeType;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Bevy3DShape {
    #[reflect(ignore)]
    pub shape: BevyShapeType,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Vec3,
    pub color: Color,
    pub unlit: bool,
}

impl Default for Bevy3DShape {
    fn default() -> Self {
        Self {
            shape: BevyShapeType::Cube,
            scale: Vec3::ONE,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            color: Color::WHITE,
            unlit: false,
        }
    }
}

type ShapeQuery<'a> = (
    Entity,
    &'a Bevy3DShape,
    Option<&'a mut Transform>,
    Option<&'a mut Mesh3d>,
    Option<&'a mut MeshMaterial3d<StandardMaterial>>,
    Ref<'a, Bevy3DShape>,
);

pub fn shapes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<ShapeQuery>,
) {
    for (entity, shape, transform_opt, mut mesh_opt, mut material_opt, shape_ref) in
        query.iter_mut()
    {
        let is_added_or_changed = shape_ref.is_changed();

        // Handle Transform
        let target_pos = shape.position;
        let target_rot = Quat::from_euler(
            EulerRot::XYZ,
            shape.rotation.x.to_radians(),
            shape.rotation.y.to_radians(),
            shape.rotation.z.to_radians(),
        );
        let target_scale = shape.scale;

        if let Some(mut transform) = transform_opt {
            if transform.translation != target_pos
                || transform.rotation != target_rot
                || transform.scale != target_scale
            {
                transform.translation = target_pos;
                transform.rotation = target_rot;
                transform.scale = target_scale;
            }
        } else {
            commands.entity(entity).insert(Transform {
                translation: target_pos,
                rotation: target_rot,
                scale: target_scale,
            });
        }

        // Handle Mesh
        if is_added_or_changed || mesh_opt.is_none() {
            let mesh_handle = match shape.shape {
                BevyShapeType::Cube => meshes.add(Cuboid::default()),
                BevyShapeType::Sphere => meshes.add(Sphere::default().mesh()),
                BevyShapeType::Capsule => meshes.add(Capsule3d::default()),
                BevyShapeType::Torus => meshes.add(Torus::default()),
                BevyShapeType::Cylinder => meshes.add(Cylinder::default()),
                BevyShapeType::Plane => meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
            };

            if let Some(mesh) = &mut mesh_opt {
                mesh.0 = mesh_handle;
            } else {
                commands.entity(entity).insert(Mesh3d(mesh_handle));
            }
        }

        // Handle Material
        if is_added_or_changed || material_opt.is_none() {
            let material_handle = materials.add(StandardMaterial {
                base_color: shape.color,
                unlit: shape.unlit,
                ..Default::default()
            });

            if let Some(mat) = &mut material_opt {
                mat.0 = material_handle;
            } else {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(material_handle));
            }
        }
    }
}
