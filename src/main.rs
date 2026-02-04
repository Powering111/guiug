use guiug::{Node, Scene, Vec3, Vec4};
use rand::Rng;

fn main() {
    // construct scene
    let mut scene = Scene::default();
    let mut rng = rand::rng();
    for i in 0..10 {
        for j in 0..10 {
            let pos = Vec3::new(0.2 * i as f32 - 0.9, 0.2 * j as f32 - 0.9, 0.0);
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

            let node = if (i + j) % 2 == 0 {
                Node::triangle(pos, size, color)
            } else {
                Node::rectangle(pos, size, color)
            };

            scene.nodes.push(node);
        }
    }

    scene.nodes.push(Node::texture(
        Vec3::new(0.0, 0.0, 0.6),
        Vec3::new(1.0, 1.0, 1.0),
        0,
    ));
    scene.nodes.push(Node::texture(
        Vec3::new(0.2, 0.2, 0.8),
        Vec3::new(1.0, 1.0, 1.0),
        0,
    ));

    // run scene
    guiug::run("wonderful program", scene);
}
