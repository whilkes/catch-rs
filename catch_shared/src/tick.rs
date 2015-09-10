use std::collections::HashMap;

use components::{Position, Orientation, LinearVelocity, PlayerState, ItemSpawn};
use event::GameEvent;
use net;

/// Stores the state of all net components in a tick
pub type ComponentsNetState<T> = HashMap<net::EntityId, T>;

#[derive(CerealData)]
pub struct NetState {
    pub position: ComponentsNetState<Position>, 
    pub orientation: ComponentsNetState<Orientation>,
    pub linear_velocity: ComponentsNetState<LinearVelocity>,
    pub player_state: ComponentsNetState<PlayerState>,
    pub item_spawn: ComponentsNetState<ItemSpawn>,

    // List of components that should not be interpolated into this tick
    // (e.g. you wouldn't want to interpolate the position of a player that was just teleported)
    pub forced_components: Vec<(net::EntityId, net::ComponentType)>,
}

#[derive(CerealData)]
pub struct Tick {
    pub tick_number: net::TickNumber,
    pub events: Vec<GameEvent>,
    pub net_state: NetState,
}

impl NetState {
    pub fn new() -> NetState {
        NetState {
            position: HashMap::new(),
            orientation: HashMap::new(),
            linear_velocity: HashMap::new(),
            player_state: HashMap::new(),
            item_spawn: HashMap::new(),

            forced_components: Vec::new(),
        }
    }
}

impl Tick {
    pub fn new(tick_number: net::TickNumber) -> Tick {
        Tick {
            tick_number: tick_number,
            events: Vec::new(),
            net_state: NetState::new(),
        }
    }
}
