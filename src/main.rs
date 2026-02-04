use guiug::{Node, Scene, Vec2};

fn main() {
    // construct scene
    let mut scene = Scene::default();
    for i in 0..10 {
        for j in 0..10 {
            let pos = Vec2::new(0.2 * i as f32 - 0.9, 0.2 * j as f32 - 0.9);
            let size = Vec2::new(0.15, 0.15);
            let node = if (i + j) % 2 == 0 {
                Node::triangle(pos, size)
            } else {
                Node::rectangle(pos, size)
            };

            scene.nodes.push(node);
        }
    }

    // run scene
    guiug::run("wonderful program", scene);
}
