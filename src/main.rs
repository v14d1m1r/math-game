#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use math_game::{Game, LEVEL_LIMITS, Mode, Operation, Phase, PlayStyle, QUESTIONS_PER_LEVEL};
use std::time::{SystemTime, UNIX_EPOCH};

const INK: Color = Color::srgb(0.16, 0.22, 0.29);
const MUTED: Color = Color::srgb(0.39, 0.45, 0.48);
const GREEN: Color = Color::srgb(0.21, 0.49, 0.36);
const CREAM: Color = Color::srgb(0.97, 0.96, 0.91);
const PEACH: Color = Color::srgb(1.0, 0.84, 0.66);
const BLUE: Color = Color::srgb(0.72, 0.84, 0.95);
const AUTO_ADVANCE_SECONDS: f32 = 2.0;
const LEVEL_NOTICE_SECONDS: u64 = 4;

#[derive(Resource)]
struct Session(Game);
#[derive(Resource, Default)]
struct AutoAdvance(Option<Timer>);
#[derive(Resource, Default)]
struct LevelNotice(Option<std::time::Duration>);
#[derive(Resource)]
struct GameFont(Handle<Font>);
#[derive(Resource)]
struct StarIcon(Handle<Image>);

impl FromWorld for StarIcon {
    fn from_world(world: &mut World) -> Self {
        let image = Image::from_buffer(
            include_bytes!("../assets/icons/star.png"),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::linear(),
            default(),
        )
        .expect("The embedded star icon must be a valid PNG");
        Self(world.resource_mut::<Assets<Image>>().add(image))
    }
}

fn star_icon(icon: &StarIcon, size: f32, color: Color) -> impl Bundle {
    (
        ImageNode {
            image: icon.0.clone(),
            color,
            ..default()
        },
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

#[derive(Component)]
struct Screen;
#[derive(Component, Clone, Copy)]
enum Action {
    Answer(u8),
    Restart,
    Mode(Mode),
    Style(PlayStyle),
}
#[derive(Component)]
struct ButtonPalette(Color);

fn seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn main() {
    App::new()
        .insert_resource(ClearColor(CREAM))
        .insert_resource(Session(Game::new(Mode::Addition, seed())))
        .init_resource::<AutoAdvance>()
        .init_resource::<LevelNotice>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Мала математика".into(),
                resolution: (960, 900).into(),
                resize_constraints: bevy::window::WindowResizeConstraints {
                    min_width: 760.0,
                    min_height: 900.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .init_resource::<StarIcon>()
        .add_systems(Startup, setup)
        .add_systems(Update, fit_window)
        .add_systems(
            Update,
            (interact, keyboard, auto_advance, level_notice, render).chain(),
        )
        .add_systems(Update, apply_font.after(render))
        .run();
}

fn setup(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../assets/fonts/LiberationSans-Regular.ttf").to_vec(),
    ));
    commands.insert_resource(GameFont(font));
    commands.spawn(Camera2d);
}

// Tiling window managers may allocate less space than the requested minimum.
// Scale the UI to keep all controls visible in that case as well.
fn fit_window(windows: Query<&Window>, mut scale: ResMut<UiScale>) {
    if let Ok(window) = windows.single() {
        let value = (window.width() / 760.0)
            .min(window.height() / 900.0)
            .min(1.25);
        if (scale.0 - value).abs() > 0.001 {
            scale.0 = value;
        }
    }
}

fn apply_font(game_font: Res<GameFont>, mut fonts: Query<&mut TextFont, Added<TextFont>>) {
    for mut font in &mut fonts {
        font.font = game_font.0.clone().into();
    }
}

fn interact(
    mut session: ResMut<Session>,
    mut buttons: Query<
        (
            &Interaction,
            Option<&Action>,
            &ButtonPalette,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, action, palette, mut background) in &mut buttons {
        *background = match interaction {
            Interaction::Hovered => palette.0.lighter(0.07).into(),
            Interaction::Pressed => palette.0.darker(0.06).into(),
            Interaction::None => palette.0.into(),
        };
        if *interaction == Interaction::Pressed {
            if let Some(action) = action {
                apply_action(&mut session, *action);
                break;
            }
        }
    }
}

fn keyboard(keys: Res<ButtonInput<KeyCode>>, mut session: ResMut<Session>) {
    for (index, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]
        .into_iter()
        .enumerate()
    {
        if keys.just_pressed(key) {
            let choice = session.0.question.choices[index];
            apply_action(&mut session, Action::Answer(choice));
            return;
        }
    }
}

fn auto_advance(time: Res<Time>, mut delay: ResMut<AutoAdvance>, mut session: ResMut<Session>) {
    if session.0.phase != Phase::Correct {
        delay.0 = None;
        return;
    }
    // Start counting on the next frame, so the success screen gets two full
    // seconds even if the frame containing the answer took a long time.
    let Some(timer) = delay.0.as_mut() else {
        delay.0 = Some(Timer::from_seconds(AUTO_ADVANCE_SECONDS, TimerMode::Once));
        return;
    };
    if timer.tick(time.delta()).is_finished() {
        session.0.advance();
        delay.0 = None;
    }
}

fn apply_action(session: &mut Session, action: Action) {
    match action {
        Action::Answer(n) => session.0.submit(n),
        Action::Restart => session.0 = Game::with_style(session.0.mode, session.0.style, seed()),
        Action::Mode(mode) => session.0 = Game::with_style(mode, session.0.style, seed()),
        Action::Style(style) => session.0 = Game::with_style(session.0.mode, style, seed()),
    }
}

fn level_notice(
    time: Res<Time>,
    session: Res<Session>,
    mut notice: ResMut<LevelNotice>,
    mut previous_level: Local<usize>,
) {
    let game = &session.0;
    if game.level > *previous_level && game.phase != Phase::Finished {
        notice.0 = Some(time.elapsed() + std::time::Duration::from_secs(LEVEL_NOTICE_SECONDS));
    } else if notice.0.is_some_and(|deadline| time.elapsed() >= deadline)
        || (notice.0.is_some() && (game.level == 0 || game.phase == Phase::Finished))
    {
        notice.0 = None;
    }
    *previous_level = game.level;
}

fn label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn column(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(gap),
        ..default()
    }
}

fn row(gap: f32) -> Node {
    Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: px(gap),
        ..default()
    }
}

fn spawn_button(
    commands: &mut Commands,
    text: impl Into<String>,
    action: Option<Action>,
    color: Color,
    width: f32,
    height: f32,
    size: f32,
) -> Entity {
    let entity = commands
        .spawn((
            Button,
            ButtonPalette(color),
            Node {
                width: px(width),
                height: px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(18)),
                ..default()
            },
            BackgroundColor(color),
            children![label(text, size, INK)],
        ))
        .id();
    // Answer buttons with no action stay visible but cannot score again.
    if let Some(action) = action {
        commands.entity(entity).insert(action);
    }
    entity
}

fn render(
    mut commands: Commands,
    session: Res<Session>,
    icon: Res<StarIcon>,
    notice: Res<LevelNotice>,
    roots: Query<Entity, With<Screen>>,
) {
    if !session.is_changed() && !notice.is_changed() {
        return;
    }
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    let game = &session.0;
    let root = commands
        .spawn((
            Screen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                padding: UiRect::all(px(20)),
                ..default()
            },
            BackgroundColor(CREAM),
        ))
        .id();
    let title = commands
        .spawn((
            column(4.0),
            children![
                label("МАЛА МАТЕМАТИКА", 16.0, GREEN),
                label("Бројимо заједно", 36.0, INK),
            ],
        ))
        .id();
    commands.entity(root).add_child(title);

    let modes = commands.spawn(row(10.0)).id();
    for mode in [Mode::Addition, Mode::Subtraction, Mode::Mixed] {
        let color = if game.mode == mode {
            Color::srgb(0.68, 0.84, 0.72)
        } else {
            Color::srgb(0.9, 0.91, 0.87)
        };
        let entity = spawn_button(
            &mut commands,
            mode.label(),
            Some(Action::Mode(mode)),
            color,
            155.0,
            42.0,
            18.0,
        );
        commands.entity(modes).add_child(entity);
    }
    commands.entity(root).add_child(modes);
    let styles = commands.spawn(row(10.0)).id();
    for style in [PlayStyle::Levels, PlayStyle::PracticeTo10] {
        let color = if game.style == style {
            Color::srgb(0.68, 0.84, 0.72)
        } else {
            Color::srgb(0.9, 0.91, 0.87)
        };
        let button = spawn_button(
            &mut commands,
            style.label(),
            Some(Action::Style(style)),
            color,
            235.0,
            38.0,
            18.0,
        );
        commands.entity(styles).add_child(button);
    }
    commands.entity(root).add_child(styles);
    let status = commands
        .spawn((
            row(45.0),
            children![
                label(
                    if game.style == PlayStyle::Levels {
                        format!("Ниво {} / 3", game.level + 1)
                    } else {
                        "Без нивоа".into()
                    },
                    22.0,
                    GREEN
                ),
                label(format!("{} бодова", game.score), 22.0, INK),
                label(format!("Бројеви до {}", game.limit()), 18.0, MUTED),
            ],
        ))
        .id();
    commands.entity(root).add_child(status);
    let progress = commands.spawn(row(12.0)).id();
    let progress_count = if game.style == PlayStyle::Levels {
        QUESTIONS_PER_LEVEL
    } else {
        0
    };
    for i in 0..progress_count {
        let done = u64::from(i) < game.completed;
        let dot = commands
            .spawn((
                Node {
                    width: px(38),
                    height: px(38),
                    border_radius: BorderRadius::MAX,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(if done {
                    PEACH
                } else {
                    Color::srgb(0.89, 0.89, 0.83)
                }),
            ))
            .id();
        let content = if done {
            commands.spawn(star_icon(&icon, 22.0, INK)).id()
        } else {
            commands.spawn(label((i + 1).to_string(), 22.0, INK)).id()
        };
        commands.entity(dot).add_child(content);
        commands.entity(progress).add_child(dot);
    }
    if game.style == PlayStyle::PracticeTo10 {
        let count = commands
            .spawn(label(
                format!("Решени задаци: {}", game.completed),
                22.0,
                GREEN,
            ))
            .id();
        commands.entity(progress).add_child(count);
    }
    commands.entity(root).add_child(progress);
    let card = commands
        .spawn((
            Node {
                width: px(680),
                min_height: px(390),
                padding: UiRect::all(px(24)),
                border_radius: BorderRadius::all(px(28)),
                ..column(16.0)
            },
            BackgroundColor(Color::WHITE),
        ))
        .id();
    commands.entity(root).add_child(card);

    if game.phase == Phase::Finished {
        let celebration = commands
            .spawn((
                column(18.0),
                children![
                    (
                        row(22.0),
                        children![
                            star_icon(&icon, 60.0, Color::srgb(0.86, 0.57, 0.16)),
                            star_icon(&icon, 60.0, Color::srgb(0.86, 0.57, 0.16)),
                            star_icon(&icon, 60.0, Color::srgb(0.86, 0.57, 0.16)),
                        ],
                    ),
                    label("Браво, мали математичару!", 32.0, GREEN),
                    label("Завршио/ла си сва 3 нивоа.", 24.0, INK),
                    label(format!("Sakupljeno: {} бодова", game.score), 28.0, INK),
                ],
            ))
            .id();
        commands.entity(card).add_child(celebration);
        let replay = spawn_button(
            &mut commands,
            "Играј поново",
            Some(Action::Restart),
            PEACH,
            260.0,
            64.0,
            24.0,
        );
        commands.entity(root).add_child(replay);
    } else {
        let equation = commands
            .spawn(label(
                format!(
                    "{}  {}  {}  =  {}",
                    game.question.left,
                    game.question.symbol(),
                    game.question.right,
                    if game.phase == Phase::Correct {
                        game.question.answer().to_string()
                    } else {
                        "?".into()
                    }
                ),
                64.0,
                INK,
            ))
            .id();
        commands.entity(card).add_child(equation);
        let instruction = match game.question.operation {
            Operation::Add => "Споји кружиће. Колико их има укупно?",
            Operation::Subtract => "Прецртани кружићи одлазе. Колико остаје?",
        };
        let explanation = if notice.0.is_some() {
            commands
                .spawn((
                    Node {
                        width: percent(100),
                        padding: UiRect::all(px(12)),
                        border_radius: BorderRadius::all(px(16)),
                        ..column(4.0)
                    },
                    BackgroundColor(GREEN),
                    children![
                        label(
                            format!("Браво! Ниво {} је откључан!", game.level + 1),
                            30.0,
                            Color::WHITE
                        ),
                        label(
                            format!("Сада бројимо до {}.", LEVEL_LIMITS[game.level]),
                            22.0,
                            Color::WHITE
                        ),
                    ],
                ))
                .id()
        } else {
            commands.spawn(label(instruction, 21.0, MUTED)).id()
        };
        commands.entity(card).add_child(explanation);
        let counters = commands.spawn(row(20.0)).id();
        match game.question.operation {
            Operation::Add => {
                let left = counter_group(&mut commands, game.question.left, 0, PEACH);
                let plus = commands.spawn(label("+", 34.0, MUTED)).id();
                let right = counter_group(&mut commands, game.question.right, 0, BLUE);
                commands.entity(counters).add_children(&[left, plus, right]);
            }
            Operation::Subtract => {
                let group = counter_group(
                    &mut commands,
                    game.question.left,
                    game.question.right,
                    PEACH,
                );
                commands.entity(counters).add_child(group);
            }
        }
        commands.entity(card).add_child(counters);
        let correct = game.phase == Phase::Correct;
        let feedback = commands
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(88),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(px(20)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(if correct { GREEN } else { Color::NONE }),
                children![label(
                    &game.feedback,
                    if correct { 46.0 } else { 25.0 },
                    if correct { Color::WHITE } else { MUTED },
                )],
            ))
            .id();
        commands.entity(card).add_child(feedback);
        let choices = commands.spawn(row(18.0)).id();
        for choice in game.question.choices {
            let tried = game.tried.contains(&choice);
            let correct = game.phase == Phase::Correct && choice == game.question.answer();
            let color = if correct {
                Color::srgb(0.64, 0.83, 0.69)
            } else if tried {
                Color::srgb(0.88, 0.88, 0.86)
            } else {
                BLUE
            };
            let action = if game.phase == Phase::Playing && !tried {
                Some(Action::Answer(choice))
            } else {
                None
            };
            let entity = spawn_button(
                &mut commands,
                choice.to_string(),
                action,
                color,
                170.0,
                86.0,
                46.0,
            );
            commands.entity(choices).add_child(entity);
        }
        commands.entity(root).add_child(choices);
        let hint = commands
            .spawn((
                Node {
                    height: px(56),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![label(
                    if correct {
                        "Следећи екран за 2 секунде…"
                    } else {
                        "Полако, имаш времена."
                    },
                    18.0,
                    MUTED,
                )],
            ))
            .id();
        commands.entity(root).add_child(hint);
    }
    let footer = commands
        .spawn(label(
            "За родитеље: промена режима или рачуна започиње нову игру.",
            15.0,
            MUTED,
        ))
        .id();
    commands.entity(root).add_child(footer);
}

fn counter_group(commands: &mut Commands, count: u8, removed: u8, color: Color) -> Entity {
    let group = commands
        .spawn(Node {
            // Five columns make larger quantities easier to count.
            width: px(192),
            min_height: px(152),
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::Center,
            row_gap: px(8),
            ..row(8.0)
        })
        .id();
    if count == 0 {
        let zero = commands.spawn(label("0", 36.0, MUTED)).id();
        commands.entity(group).add_child(zero);
    }
    for i in 0..count {
        let crossed = i >= count - removed;
        let dot = commands
            .spawn((
                Node {
                    width: px(32),
                    height: px(32),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::MAX,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if crossed {
                    Color::srgb(0.9, 0.9, 0.88)
                } else {
                    color
                }),
            ))
            .id();
        if crossed {
            let cross = commands.spawn(label("×", 28.0, MUTED)).id();
            commands.entity(dot).add_child(cross);
        }
        commands.entity(group).add_child(dot);
    }
    group
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.insert_resource(Session(Game::new(Mode::Addition, 42)))
            .init_resource::<Assets<Image>>()
            .init_resource::<StarIcon>()
            .init_resource::<Time>()
            .init_resource::<AutoAdvance>()
            .init_resource::<LevelNotice>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(
                Update,
                (interact, keyboard, auto_advance, level_notice, render).chain(),
            );
        app.update();
        app
    }

    fn press(app: &mut App, matches: impl Fn(&Action) -> bool) {
        let world = app.world_mut();
        let entity = world
            .query::<(Entity, &Action)>()
            .iter(world)
            .find(|(_, action)| matches(action))
            .unwrap()
            .0;
        *world.get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
        world
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::ZERO);
        app.update();
    }

    fn elapse(app: &mut App, milliseconds: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(milliseconds));
        app.update();
    }

    #[test]
    fn buttons_support_retry_next_and_mode_reset_without_leaking_screens() {
        let mut app = test_app();
        let answer = app.world().resource::<Session>().0.question.answer();
        press(
            &mut app,
            |action| matches!(action, Action::Answer(n) if *n != answer),
        );
        assert_eq!(app.world().resource::<Session>().0.tried.len(), 1);
        press(
            &mut app,
            |action| matches!(action, Action::Answer(n) if *n == answer),
        );
        assert_eq!(app.world().resource::<Session>().0.score, 5);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Correct);
        elapse(&mut app, 1_999);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Correct);
        elapse(&mut app, 1);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Playing);
        press(&mut app, |action| {
            matches!(action, Action::Mode(Mode::Subtraction))
        });
        let game = &app.world().resource::<Session>().0;
        assert_eq!(game.mode, Mode::Subtraction);
        assert_eq!(game.score, 0);
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<Screen>>()
                .iter(world)
                .count(),
            1
        );
        assert_eq!(
            world
                .query::<&Text>()
                .iter(world)
                .filter(|text| text.0 == "Бројимо заједно")
                .count(),
            1
        );
    }

    #[test]
    fn keyboard_selects_choice_and_advances_after_success() {
        let mut app = test_app();
        let game = &app.world().resource::<Session>().0;
        let index = game
            .question
            .choices
            .iter()
            .position(|&n| n == game.question.answer())
            .unwrap();
        let key = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3][index];
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
        assert_eq!(app.world().resource::<Session>().0.score, 10);
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        elapse(&mut app, 2_000);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Playing);
        assert_eq!(app.world().resource::<Session>().0.completed, 1);
    }

    #[test]
    fn auto_advance_handles_level_changes_and_finishes_the_game() {
        let mut app = test_app();
        for question in 0..15 {
            let answer = app.world().resource::<Session>().0.question.answer();
            press(
                &mut app,
                |action| matches!(action, Action::Answer(n) if *n == answer),
            );
            elapse(&mut app, 2_000);
            let game = &app.world().resource::<Session>().0;
            if question == 14 {
                assert_eq!(game.phase, Phase::Finished);
            } else {
                assert_eq!(game.phase, Phase::Playing);
                assert_eq!(game.level, (question + 1) / 5);
            }
        }
        elapse(&mut app, 10_000);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Finished);
        assert_eq!(app.world().resource::<Session>().0.score, 150);
    }

    #[test]
    fn progress_and_reward_use_embedded_star_images() {
        let mut app = test_app();
        let handle = app.world().resource::<StarIcon>().0.clone();
        let image = app
            .world()
            .resource::<Assets<Image>>()
            .get(&handle)
            .unwrap();
        assert_eq!(image.size(), UVec2::splat(128));
        let pixels = image.data.as_ref().unwrap();
        assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] == 255));
        assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] == 0));

        for question in 0..15 {
            let answer = app.world().resource::<Session>().0.question.answer();
            press(
                &mut app,
                |action| matches!(action, Action::Answer(value) if *value == answer),
            );
            if question == 0 {
                let world = app.world_mut();
                assert_eq!(
                    world
                        .query::<&ImageNode>()
                        .iter(world)
                        .filter(|image| image.image == handle)
                        .count(),
                    1
                );
            }
            elapse(&mut app, 2_000);
        }
        let world = app.world_mut();
        assert_eq!(
            world
                .query::<&ImageNode>()
                .iter(world)
                .filter(|image| image.image == handle)
                .count(),
            8
        );
        assert!(
            !world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0.contains('★'))
        );
    }

    #[test]
    fn level_notice_appears_for_each_new_level_and_expires() {
        let mut app = test_app();
        assert!(app.world().resource::<LevelNotice>().0.is_none());
        for question in 0..15 {
            let answer = app.world().resource::<Session>().0.question.answer();
            press(
                &mut app,
                |action| matches!(action, Action::Answer(value) if *value == answer),
            );
            elapse(&mut app, 2_000);
            if question == 4 || question == 9 {
                let level = app.world().resource::<Session>().0.level + 1;
                let message = format!("Браво! Ниво {level} је откључан!");
                let world = app.world_mut();
                assert!(
                    world
                        .query::<&Text>()
                        .iter(world)
                        .any(|text| text.0 == message)
                );
                elapse(&mut app, 3_999);
                assert!(app.world().resource::<LevelNotice>().0.is_some());
                elapse(&mut app, 1);
                assert!(app.world().resource::<LevelNotice>().0.is_none());
                let world = app.world_mut();
                assert!(
                    !world
                        .query::<&Text>()
                        .iter(world)
                        .any(|text| text.0 == message)
                );
            } else {
                assert!(app.world().resource::<LevelNotice>().0.is_none());
            }
        }
    }

    #[test]
    fn practice_selection_preserves_operation_and_never_advances_levels() {
        let mut app = test_app();
        press(&mut app, |action| {
            matches!(action, Action::Mode(Mode::Subtraction))
        });
        press(&mut app, |action| {
            matches!(action, Action::Style(PlayStyle::PracticeTo10))
        });
        assert_eq!(app.world().resource::<Session>().0.mode, Mode::Subtraction);
        for _ in 0..16 {
            let answer = app.world().resource::<Session>().0.question.answer();
            press(
                &mut app,
                |action| matches!(action, Action::Answer(value) if *value == answer),
            );
            elapse(&mut app, 2_000);
            assert_eq!(app.world().resource::<Session>().0.phase, Phase::Playing);
            assert!(app.world().resource::<LevelNotice>().0.is_none());
        }
        let world = app.world_mut();
        assert!(
            world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0 == "Решени задаци: 16")
        );
        assert!(
            !world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0.starts_with("Ниво "))
        );
        press(&mut app, |action| {
            matches!(action, Action::Mode(Mode::Mixed))
        });
        let game = &app.world().resource::<Session>().0;
        assert_eq!(game.style, PlayStyle::PracticeTo10);
        assert_eq!(game.score, 0);
        assert_eq!(game.completed, 0);
        press(&mut app, |action| {
            matches!(action, Action::Style(PlayStyle::Levels))
        });
        let game = &app.world().resource::<Session>().0;
        assert_eq!(game.style, PlayStyle::Levels);
        assert_eq!(game.mode, Mode::Mixed);
        assert_eq!(game.level, 0);
    }

    #[test]
    fn changing_mode_dismisses_level_notice() {
        let mut app = test_app();
        for _ in 0..5 {
            let answer = app.world().resource::<Session>().0.question.answer();
            press(
                &mut app,
                |action| matches!(action, Action::Answer(value) if *value == answer),
            );
            elapse(&mut app, 2_000);
        }
        assert!(app.world().resource::<LevelNotice>().0.is_some());
        press(&mut app, |action| {
            matches!(action, Action::Mode(Mode::Subtraction))
        });
        assert!(app.world().resource::<LevelNotice>().0.is_none());
    }

    #[test]
    fn switching_to_practice_cancels_level_notice_and_pending_advance() {
        let mut app = test_app();
        for _ in 0..5 {
            let answer = app.world().resource::<Session>().0.question.answer();
            press(
                &mut app,
                |action| matches!(action, Action::Answer(value) if *value == answer),
            );
            elapse(&mut app, 2_000);
        }
        assert!(app.world().resource::<LevelNotice>().0.is_some());
        let answer = app.world().resource::<Session>().0.question.answer();
        press(
            &mut app,
            |action| matches!(action, Action::Answer(value) if *value == answer),
        );
        press(&mut app, |action| {
            matches!(action, Action::Style(PlayStyle::PracticeTo10))
        });
        elapse(&mut app, 5_000);
        assert!(app.world().resource::<LevelNotice>().0.is_none());
        assert!(app.world().resource::<AutoAdvance>().0.is_none());
        assert_eq!(app.world().resource::<Session>().0.completed, 0);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Playing);
    }

    #[test]
    fn switching_mode_cancels_the_pending_auto_advance() {
        let mut app = test_app();
        let answer = app.world().resource::<Session>().0.question.answer();
        press(
            &mut app,
            |action| matches!(action, Action::Answer(n) if *n == answer),
        );
        elapse(&mut app, 1_000);
        press(&mut app, |action| {
            matches!(action, Action::Mode(Mode::Subtraction))
        });
        elapse(&mut app, 3_000);
        assert_eq!(app.world().resource::<Session>().0.completed, 0);
        let answer = app.world().resource::<Session>().0.question.answer();
        press(
            &mut app,
            |action| matches!(action, Action::Answer(n) if *n == answer),
        );
        elapse(&mut app, 1_000);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Correct);
        elapse(&mut app, 1_000);
        assert_eq!(app.world().resource::<Session>().0.phase, Phase::Playing);
    }
}
