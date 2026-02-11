use std::sync::{Arc, Mutex};

use guiug::{Anchor, Event, Guiug, KeyCode, PhysicalKey, Position, Size, TextAnchor, Vec4};

fn main() {
    let counter_num = Arc::new(Mutex::new(0i64));

    let mut guiug = Guiug::default();

    // texture info
    let awesomeface_texture = guiug.add_texture(include_bytes!("res/awesomeface_3d.png"));
    let ldmsys_texture = guiug.add_texture(include_bytes!("res/ldmsys.png"));
    let demisoda_texture = guiug.add_texture(include_bytes!("res/demisoda.jpg"));
    let library_texture = guiug.add_texture(include_bytes!("res/kaist_library.jpg"));
    let gamma_texture = guiug.add_texture(include_bytes!("res/gamma-ramp32.png"));

    // font info
    let arial_font = guiug.add_font(include_bytes!("res/arial.ttf"));
    let malgun_font = guiug.add_font(include_bytes!("res/malgun.ttf"));

    // construct scene
    let mut root = Vec::new();

    let happy_day_node = guiug.text_node(
        "즐거운 하루!!!".to_owned(),
        malgun_font,
        Size::ParentHeight(0.15),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
        TextAnchor::Center(Size::ZERO),
        TextAnchor::Center(Size::ZERO),
    );

    let counter_node = guiug.text_node(
        "Counter: 0".to_owned(),
        arial_font,
        Size::Pixel(40),
        Vec4::new(1.0, 0.2, 0.2, 1.0),
        TextAnchor::End(Size::ZERO),
        TextAnchor::End(Size::Pixel(10)),
    );

    root.extend([
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::end(Size::ParentHeight(0.4), Size::ScreenWidth(0.2)),
            ),
            guiug.texture_node(awesomeface_texture),
        ),
        (
            Position::new(
                Anchor::end(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::end(Size::ParentHeight(0.4), Size::ScreenWidth(0.2)),
            ),
            guiug.texture_node(ldmsys_texture),
        ),
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::end(Size::ParentHeight(0.1), Size::ScreenWidth(0.2)),
            ),
            guiug.texture_node(demisoda_texture),
        ),
        (
            Position::new(
                Anchor::end(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::end(Size::ParentHeight(0.1), Size::ScreenWidth(0.2)),
            ),
            guiug.texture_node(library_texture),
        ),
        (
            Position::new(
                Anchor::stretch(Size::Pixel(100), Size::Pixel(100)),
                Anchor::start(Size::ParentHeight(0.1), Size::ParentHeight(0.2)),
            ),
            guiug.texture_node(gamma_texture),
        ),
        (
            Position::new(
                Anchor::end(Size::ZERO, Size::Pixel(400)),
                Anchor::start(Size::ZERO, Size::Pixel(400)),
            ),
            guiug.rect_node(Vec4::new(1.0, 1.0, 1.0, 1.0)),
        ),
        (
            Position::new(
                Anchor::end(Size::ZERO, Size::Pixel(100)),
                Anchor::end(Size::ZERO, Size::Pixel(100)),
            ),
            counter_node,
        ),
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::start(Size::ParentHeight(0.4), Size::ScreenWidth(0.2)),
            ),
            guiug.text_node(
                "Hello world!".to_owned(),
                arial_font,
                Size::Pixel(40),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                TextAnchor::Start(Size::ZERO),
                TextAnchor::Center(Size::ZERO),
            ),
        ),
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::start(Size::ParentHeight(0.5), Size::ScreenWidth(0.2)),
            ),
            guiug.text_node(
                "Bye world!".to_owned(),
                arial_font,
                Size::Pixel(40),
                Vec4::new(1.0, 0.0, 1.0, 1.0),
                TextAnchor::Start(Size::Pixel(20)),
                TextAnchor::Center(Size::ZERO),
            ),
        ),
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::start(Size::ParentHeight(0.6), Size::ScreenWidth(0.2)),
            ),
            {
                let inner_vec = vec![
                    (
                        Position::FULL,
                        guiug.rect_node(Vec4::new(0.2, 0.9, 0.5, 1.0)),
                    ),
                    (Position::FULL, happy_day_node),
                ];
                guiug.layer_node(inner_vec)
            },
        ),
    ]);

    // small rectangles
    for y in 0..100 {
        for x in 0..100 {
            let node = guiug.rect_node(Vec4::new(
                (y % 100) as f32 / 100.0,
                (x % 100) as f32 / 100.0,
                0.0,
                1.0,
            ));
            root.push((
                Position::new(
                    Anchor::end(Size::Pixel(x * 4), Size::Pixel(3)),
                    Anchor::start(Size::Pixel(y * 4), Size::Pixel(3)),
                ),
                node,
            ));
        }
    }

    let button_hitbox = guiug.hitbox_node();
    let button_node = vec![
        (
            Position::FULL,
            guiug.rect_node(Vec4::new(0.9, 0.9, 0.9, 1.0)),
        ),
        (
            Position::FULL,
            guiug.text_node(
                "Click me!".to_owned(),
                arial_font,
                Size::ParentHeight(0.4),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
                TextAnchor::Center(Size::ZERO),
                TextAnchor::Center(Size::ParentHeight(0.1)),
            ),
        ),
        (Position::FULL, button_hitbox),
    ];
    let button_hitbox2 = guiug.hitbox_node();
    let button_node2 = vec![
        (
            Position::FULL,
            guiug.rect_node(Vec4::new(0.9, 0.9, 0.9, 1.0)),
        ),
        (
            Position::FULL,
            guiug.text_node(
                "Click me! 2".to_owned(),
                arial_font,
                Size::ParentHeight(0.4),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
                TextAnchor::Center(Size::ZERO),
                TextAnchor::Center(Size::ParentHeight(0.1)),
            ),
        ),
        (Position::FULL, button_hitbox2),
    ];

    root.push((
        Position::new(
            Anchor::start(Size::Pixel(300), Size::Pixel(200)),
            Anchor::center(Size::ZERO, Size::Pixel(100)),
        ),
        guiug.layer_node(button_node2),
    ));
    root.push((
        Position::new(
            Anchor::start(Size::Pixel(200), Size::Pixel(200)),
            Anchor::center(Size::Pixel(50), Size::Pixel(100)),
        ),
        guiug.layer_node(button_node),
    ));

    let root_node = guiug.layer_node(root);
    guiug.set_root(root_node);

    // Interaction for changing happiness
    let mut is_happy = true;
    guiug.interaction(
        Event::KeyDown(PhysicalKey::Code(KeyCode::Space)),
        |runtime| {
            let node = runtime.get_node_mut(&happy_day_node).unwrap();
            if let guiug::Node::Text { text, size, .. } = node {
                is_happy = !is_happy;
                if is_happy {
                    *text = "즐거운 하루!!!".to_owned();
                    *size = Size::ParentHeight(0.15);
                } else {
                    *text = "즐겁지 않은 하루...".to_owned();
                    *size = Size::ParentHeight(0.12);
                }
            }
        },
    );

    // Interaction for updating counter
    let update_counter = |runtime: &mut guiug::Runtime, count: i64| {
        let node = runtime.get_node_mut(&counter_node).unwrap();
        let mut counter_num = counter_num.lock().unwrap();

        *counter_num += count;
        if let guiug::Node::Text { text, .. } = node {
            *text = format!("Counter: {}", counter_num);
        }
    };
    guiug.interaction(
        Event::KeyDown(PhysicalKey::Code(KeyCode::ArrowUp)),
        |runtime| {
            update_counter(runtime, 1);
        },
    );

    guiug.interaction(
        Event::KeyDown(PhysicalKey::Code(KeyCode::ArrowDown)),
        |runtime| {
            update_counter(runtime, -1);
        },
    );

    // Interaction for button
    guiug.interaction(Event::Click(button_hitbox), |runtime| {
        update_counter(runtime, 100);
    });
    guiug.interaction(Event::Click(button_hitbox2), |runtime| {
        update_counter(runtime, -100);
    });

    // Exit on esc
    guiug.interaction(
        Event::KeyDown(PhysicalKey::Code(KeyCode::Escape)),
        |runtime| {
            runtime.exit();
        },
    );

    // run scene
    guiug::run("wonderful program", guiug);
}
