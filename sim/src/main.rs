pub mod sim;

use sim::Sim;

fn main() {
    let mut sim = Sim::init();

    while !sim.rl_handle.window_should_close() {
        sim.update();
        sim.draw();
    }
}

