use guiug::{Anchor, Guiug, Position, Size, Vec4};

fn main() {
    let mut guiug = Guiug::default();

    // texture info
    let awesomeface_texture = guiug.add_texture(include_bytes!("res/awesomeface_3d.png"));
    let ldmsys_texture = guiug.add_texture(include_bytes!("res/ldmsys.png"));
    let demisoda_texture = guiug.add_texture(include_bytes!("res/demisoda.jpg"));
    let library_texture = guiug.add_texture(include_bytes!("res/kaist_library.jpg"));
    let gamma_texture = guiug.add_texture(include_bytes!("res/gamma-ramp32.png"));

    // construct scene
    let mut root = vec![
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
    ];

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
