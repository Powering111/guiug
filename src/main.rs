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
                Anchor::start(Size::Ratio(0.2), Size::Pixel(200)),
                Anchor::end(Size::Ratio(0.4), Size::Pixel(200)),
            ),
            guiug.texture_node(awesomeface_texture),
        ),
        (
            Position::new(
                Anchor::end(Size::Ratio(0.2), Size::Pixel(200)),
                Anchor::end(Size::Ratio(0.4), Size::Pixel(200)),
            ),
            guiug.texture_node(ldmsys_texture),
        ),
        (
            Position::new(
                Anchor::start(Size::Ratio(0.2), Size::Pixel(200)),
                Anchor::end(Size::Ratio(0.1), Size::Pixel(200)),
            ),
            guiug.texture_node(demisoda_texture),
        ),
        (
            Position::new(
                Anchor::end(Size::Ratio(0.2), Size::Pixel(200)),
                Anchor::end(Size::Ratio(0.1), Size::Pixel(200)),
            ),
            guiug.texture_node(library_texture),
        ),
        (
            Position::new(
                Anchor::stretch(Size::Pixel(100), Size::Pixel(100)),
                Anchor::start(Size::Ratio(0.1), Size::Ratio(0.2)),
            ),
            guiug.texture_node(gamma_texture),
        ),
    ];

    for i in 0..10 {
        for j in 0..10 {
            let color = Vec4::new(0.1 * i as f32, 0.1 * j as f32, 0.0, 1.0);
            root.push((
                Position::new(
                    Anchor::start(Size::Ratio(0.1 * i as f32 + 0.02), Size::Ratio(0.06)),
                    Anchor::start(Size::Ratio(0.1 * j as f32 + 0.02), Size::Ratio(0.06)),
                ),
                guiug.rect_node(color),
            ));
        }
    }

    let root_node = guiug.layer_node(root);
    guiug.set_root(root_node);

    // run scene
    guiug::run("wonderful program", guiug);
}
