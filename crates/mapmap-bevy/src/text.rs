use crate::components::{Bevy3DText, BevyTextAlignment};
use bevy::prelude::*;
use bevy::text::{JustifyText, Text2d, TextColor, TextFont, TextLayout};

pub fn text_system(
    mut commands: Commands,
    query: Query<(Entity, &Bevy3DText), Changed<Bevy3DText>>,
) {
    for (entity, config) in query.iter() {
        let justify = match config.alignment {
            BevyTextAlignment::Left => JustifyText::Left,
            BevyTextAlignment::Center => JustifyText::Center,
            BevyTextAlignment::Right => JustifyText::Right,
            BevyTextAlignment::Justify => JustifyText::Justified,
        };

        let color = Color::srgba(
            config.color[0],
            config.color[1],
            config.color[2],
            config.color[3],
        );

        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            config.rotation[0].to_radians(),
            config.rotation[1].to_radians(),
            config.rotation[2].to_radians(),
        );

        commands.entity(entity).insert((
            Text::new(config.text.clone()),
            TextFont {
                font_size: config.font_size,
                ..default()
            },
            TextColor(color),
            TextLayout {
                justify,
                ..default()
            },
            Text2d::default(),
            Transform::from_xyz(config.position[0], config.position[1], config.position[2])
                .with_rotation(rotation),
        ));
    }
}
