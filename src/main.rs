use guiug::{Guiug, Position, UVec2, Vec4};
use rand::Rng;

fn main() {
    let mut rng = rand::rng();
    let mut guiug = Guiug::default();

    // texture info
    let icon_texture = guiug.add_texture(include_bytes!("res/awesomeface_3d.png"));
    let ldmsys_texture = guiug.add_texture(include_bytes!("res/ldmsys.png"));
    let gamma_texture = guiug.add_texture(include_bytes!("res/gamma-ramp32.png"));

    // construct scene
    let mut root = Vec::new();

    let awesomeface_node = guiug.texture_node(icon_texture);
    root.push((
        Position::Absolute {
            position: UVec2::new(300, 100),
            size: UVec2::new(300, 300),
        },
        awesomeface_node,
    ));
    let ldmsys_node = guiug.texture_node(ldmsys_texture);
    root.push((
        Position::Absolute {
            position: UVec2::new(600, 200),
            size: UVec2::new(400, 400),
        },
        ldmsys_node,
    ));
    let gamma_node = guiug.texture_node(gamma_texture);
    root.push((
        Position::Absolute {
            position: UVec2::new(100, 700),
            size: UVec2::new(800, 200),
        },
        gamma_node,
    ));

    for i in 0..10 {
        for j in 0..10 {
            let position = UVec2::new(100 * i + 20, 100 * j + 20);
            let size = UVec2::new(60, 60);
            let color = [
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
                Vec4::new(0.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 0.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 0.0, 1.0),
            ][rng.random_range(0..7) as usize];

            let node = guiug.rect_node(color);
            root.push((Position::Absolute { position, size }, node));
        }
    }

    let root_node = guiug.layer_node(root);
    guiug.set_root(root_node);

    // run scene
    guiug::run("wonderful program", guiug);
}
