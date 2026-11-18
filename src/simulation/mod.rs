mod gravity;
mod planets;
mod trajectory;

pub use gravity::{
    CentralBody, GRAVITY_CONSTANT, GravityPlugin, Mass, SimulationSpeed, SimulationState, Velocity,
};
pub use planets::PlanetsPlugin;
pub use trajectory::{TrajectoryMode, TrajectoryPlugin};
