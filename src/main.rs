use guiug::{Anchor, Guiug, Position, Size, Vec4};

fn main() {
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

    root.extend([
        (
            Position::new(
                Anchor::end(Size::ZERO, Size::Pixel(400)),
                Anchor::start(Size::ZERO, Size::Pixel(400)),
            ),
            guiug.rect_node(Vec4::new(1.0, 1.0, 1.0, 1.0)),
        ),
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::start(Size::ParentHeight(0.4), Size::ScreenWidth(0.2)),
            ),
            guiug.text_node(
                "Hello world!".to_owned(),
                arial_font,
                40,
                Vec4::new(1.0, 1.0, 1.0, 1.0),
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
                40,
                Vec4::new(1.0, 0.0, 1.0, 1.0),
            ),
        ),
        (
            Position::new(
                Anchor::start(Size::ParentWidth(0.2), Size::ScreenWidth(0.2)),
                Anchor::start(Size::ParentHeight(0.6), Size::ScreenWidth(0.2)),
            ),
            guiug.text_node(
                "즐거운 하루!!!".to_owned(),
                malgun_font,
                60,
                Vec4::new(0.0, 0.0, 1.0, 1.0),
            ),
        ),
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
    ]);

    // Tile rectangles
    let mut row_vec = Vec::new();
    for i in 0..10 {
        let mut col_vec = Vec::new();
        for j in 0..10 {
            let color = Vec4::new(0.1 * i as f32, 0.1 * j as f32, 0.0, 1.0);
            let rect_node = guiug.rect_node(color);
            // layer node for margin
            let layer_node = guiug.layer_node(vec![(
                Position::new(
                    Anchor::center(Size::ZERO, Size::ParentWidth(0.8)),
                    Anchor::center(Size::ZERO, Size::ParentHeight(0.8)),
                ),
                rect_node,
            )]);
            col_vec.push((Size::Weight(1.0), layer_node));
        }
        row_vec.push((Size::Weight(1.0), guiug.column_node(col_vec)));
    }
    root.push((Position::FULL, guiug.row_node(row_vec)));

    // Row & Column demonstration
    let col_vec = vec![
        (
            Size::Weight(1.0),
            guiug.rect_node(Vec4::new(0.0, 1.0, 1.0, 1.0)),
        ),
        (
            Size::Weight(1.0),
            guiug.rect_node(Vec4::new(1.0, 1.0, 1.0, 1.0)),
        ),
        (
            Size::Weight(1.0),
            guiug.rect_node(Vec4::new(0.0, 1.0, 1.0, 1.0)),
        ),
        (Size::Weight(2.0), guiug.empty_node()),
        (
            Size::Weight(1.0),
            guiug.rect_node(Vec4::new(0.0, 1.0, 1.0, 1.0)),
        ),
    ];

    let row_vec = vec![
        (
            Size::Pixel(100),
            guiug.rect_node(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        ),
        (Size::Weight(1.0), guiug.column_node(col_vec.clone())),
        (
            Size::Weight(1.0),
            guiug.rect_node(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        ),
        (Size::Weight(2.0), guiug.empty_node()),
        (
            Size::Weight(1.0),
            guiug.rect_node(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        ),
    ];

    root.push((Position::FULL, guiug.row_node(row_vec)));

    let root_node = guiug.layer_node(root);
    guiug.set_root(root_node);

    // run scene
    guiug::run("wonderful program", guiug);
}
