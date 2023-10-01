use crate::input::SelectedBuilding;
use crate::level::{validate_solution, CellType, Position, Puzzle, Solution};
use crate::GameState;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::ui::RelativeCursorPosition;
use std::default::Default;

pub const CELL_SIZE: f32 = 100.0;

#[derive(Component, Default)]
pub struct LevelRender {
    field: Vec<Vec<Entity>>,
    placements: Vec<Entity>,
}

#[derive(Component)]
pub struct SolutionStatusText;

#[derive(Component)]
pub struct AvailableBuildingsText {
    building_index: usize,
}

pub fn create_level_render(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut level_render_query: Query<(Entity, &mut LevelRender)>,
    server: Res<AssetServer>,
) {
    let (level_render_entity, mut level_render) = level_render_query.single_mut();
    let puzzle = &game_state.puzzle;

    let (rows, columns) = (puzzle.rows(), puzzle.columns());
    level_render.field.resize(rows, vec![]);
    for r in 0..rows {
        for c in 0..columns {
            let color = if puzzle.field[r][c] == CellType::Grass {
                Color::Rgba {
                    alpha: 1.0,
                    blue: 133.0 / 256.0,
                    green: 242.0 / 256.0,
                    red: 173.0 / 256.0,
                }
            } else {
                Color::NONE
            };
            let id = commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        anchor: Anchor::BottomLeft,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(c as f32 * CELL_SIZE, r as f32 * CELL_SIZE, 0.0),
                    ..Default::default()
                })
                .id();
            commands.entity(level_render_entity).add_child(id);
            level_render.field[r].push(id);
        }
    }

    for placement in game_state.solution.placements.iter() {
        let id = commands
            .spawn(SpriteBundle {
                texture: server.load(placement.building.get_asset_name()),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    anchor: Anchor::BottomLeft,
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                ..Default::default()
            })
            .id();
        commands.entity(level_render_entity).add_child(id);
        level_render.placements.push(id);
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(20.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for (index, building) in game_state.puzzle.building_count.keys().enumerate() {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(10.),
                            height: Val::Percent(100.),
                            margin: UiRect::right(Val::Px(50.)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(ImageBundle {
                                image: UiImage {
                                    texture: server.load(building.get_asset_name()),
                                    flip_x: false,
                                    flip_y: false,
                                },
                                style: Style {
                                    width: Val::Px(100.),
                                    height: Val::Px(100.),
                                    margin: UiRect::top(Val::Px(20.)),
                                    ..default()
                                },
                                ..default()
                            })
                            .insert(RelativeCursorPosition::default());

                        parent.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font_size: 24.0,
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                            ),
                            AvailableBuildingsText {
                                building_index: index,
                            },
                        ));
                    });
            }
        });

    commands.spawn((
        TextBundle::from_section(
            "Solution status:",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..Default::default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            width: Val::Px(600.0),
            ..default()
        }),
        SolutionStatusText,
    ));
}

pub fn destroy_level_render(
    mut commands: Commands,
    level_render_query: Query<Entity, (With<LevelRender>, With<Transform>)>,
) {
    let level_render_entity = level_render_query.single();
    commands.entity(level_render_entity).despawn_descendants();
    commands.entity(level_render_entity).clear_children();
}

pub fn update_level_render(
    game_state: Res<GameState>,
    mut level_render_query: Query<(Entity, &LevelRender, &mut Transform)>,
) {
    let (_, _, mut transform) = level_render_query.single_mut();
    let puzzle = &game_state.puzzle;
    let (rows, columns) = (puzzle.rows(), puzzle.columns());
    let (puzzle_width, puzzle_height) = (columns as f32 * CELL_SIZE, rows as f32 * CELL_SIZE);
    transform.translation = Vec3::new(-puzzle_width / 2.0, -puzzle_height / 2.0, 0.0);
}

pub fn update_placements_render(
    game_state: Res<GameState>,
    level_render_query: Query<&LevelRender>,
    mut sprites_query: Query<(&mut Transform, &mut Visibility)>,
) {
    let level_render = level_render_query.single();

    for i in 0..game_state.solution.placements.len() {
        let placement = &game_state.solution.placements[i];
        let id = level_render.placements[i];
        if let Ok((mut transform, mut visibility)) = sprites_query.get_mut(id) {
            let position = placement.position.unwrap_or(Position { row: 0, column: 0 });
            let visible = placement.position.is_some();
            *transform = Transform::from_xyz(
                position.column as f32 * CELL_SIZE,
                position.row as f32 * CELL_SIZE,
                0.0,
            );
            *visibility = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn build_available_buildings_texts(puzzle: &Puzzle, solution: &Solution) -> Vec<String> {
    let placed_building_count = solution.building_count();
    let mut messages = Vec::new();
    for (index, (building, total_count)) in puzzle.building_count.iter().enumerate() {
        let placed_count = placed_building_count
            .get(building)
            .cloned()
            .unwrap_or_default();
        messages.push(format!(
            "{}: {building:?}: {placed_count}/{total_count}",
            index + 1
        ));
    }
    messages
}

// TODO: We can actually update this only if solution changes.
pub fn update_solution_status(
    game_state: Res<GameState>,
    mut solution_status_text_query: Query<&mut Text, With<SolutionStatusText>>,
) {
    let validation_result = validate_solution(&game_state.solution, &game_state.puzzle);
    solution_status_text_query.single_mut().sections[0].value = format!("{}", validation_result);
}

// TODO: We can actually update this only if solution changes.
pub fn update_available_buildings(
    game_state: Res<GameState>,
    selected_building: Res<SelectedBuilding>,
    mut available_buildings_text: Query<(&mut Text, &AvailableBuildingsText)>,
) {
    let messages = build_available_buildings_texts(&game_state.puzzle, &game_state.solution);
    for (mut text, available_building_text_component) in available_buildings_text.iter_mut() {
        text.sections[0].value = messages[available_building_text_component.building_index].clone();
        if selected_building.number == Some(available_building_text_component.building_index) {
            text.sections[0].style.color = Color::RED;
        } else {
            text.sections[0].style.color = Color::WHITE;
        }
    }
}
