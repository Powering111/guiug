use guiug::{Guiug, Node, Scene, Vec3, Vec4};
use rand::Rng;

fn main() {
    let mut guiug = Guiug::default();

    // texture info
    let icon_texture = guiug.add_texture(include_bytes!("res/awesomeface_3d.png"));
    let icon_texture2 = guiug.add_texture(include_bytes!("res/ldmsys.png"));
    assert_eq!(icon_texture, 0);
    assert_eq!(icon_texture2, 1);

    // construct scene
    let mut scene = Scene::default();
    let mut rng = rand::rng();
    for i in 0..10 {
        for j in 0..10 {
            let pos = Vec3::new(0.2 * i as f32 - 0.9, 0.2 * j as f32 - 0.9, 0.9);
            let size = Vec3::new(0.15, 0.15, 1.0);
            let color = [
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
                Vec4::new(0.0, 0.0, 1.0, 1.0),
                Vec4::new(0.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 0.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 0.0, 1.0),
            ][rng.random_range(0..7) as usize];

            let node = Node::rectangle(pos, size, color);

            scene.nodes.push(node);
        }
    }

    scene.nodes.push(Node::texture(
        Vec3::new(-0.4, -0.4, 0.6),
        Vec3::new(1.0, 1.0, 1.0),
        icon_texture,
    ));
    scene.nodes.push(Node::texture(
        Vec3::new(0.4, 0.4, 0.4),
        Vec3::new(1.0, 1.0, 1.0),
        icon_texture2,
    ));

    guiug.set_scene(scene);

    // run scene
    guiug::run("wonderful program", guiug);
}
