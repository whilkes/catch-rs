use std::collections::HashMap;

use ecs::ServiceManager;

use shared::{EntityId, EntityTypeId, EntityTypes, TickNumber, PlayerId, GameEvent};
use shared::services::HasEvents;
use shared::entities::NetEntities;

// State that can be accessed mutably by systems
pub struct Services {
    // List of entity types by name
    pub entity_types: EntityTypes,

    // Tick duration in seconds
    pub tick_dur_s: f32,

    // Events generated in a tick 
    pub next_events: Vec<GameEvent>,
    
    // Game events for the current tick that are to be sent to clients are stored in
    // `next_player_events`.  Each event in `next_events` is also stored for each player here.
    pub next_player_events: HashMap<PlayerId, Vec<GameEvent>>,

    // Net entities
    pub net_entities: NetEntities,

    // Counter for creating net entities
    entity_id_counter: EntityId,
}

impl HasEvents for Services {
    /// Queue event for every player and also execute it on the server
    fn add_event(&mut self, event: &GameEvent) {
        let player_ids = self.next_player_events.keys().map(|k| *k)
                             .collect::<Vec<_>>();

        for player_id in player_ids.iter() {
            self.next_player_events.get_mut(player_id).unwrap()
                .push(event.clone());
        }

        self.next_events.push(event.clone());
    }
}

impl Services {
    pub fn new(entity_types: EntityTypes) -> Services {
        Services {
            entity_types: entity_types,
            tick_dur_s: 0.0, // the correct duration is set by GameState::tick
            next_events: Vec::new(),
            next_player_events: HashMap::new(),
            net_entities: NetEntities::default(),
            entity_id_counter: 0,
        }
    }

    pub fn prepare_for_tick<T: Iterator<Item=PlayerId>>
                           (&mut self,
                            _number: TickNumber,
                            player_ids: T) {
        assert!(self.next_events.is_empty());

        let mut next_player_events = HashMap::new();
        for player_id in player_ids {
            next_player_events.insert(player_id, Vec::new()); 
        }

        // Right now, we don't want to allow queueing events for a player before the tick starts
        for (id, _) in self.next_player_events.iter() {
            if next_player_events.get(id).is_some() {
                assert!(self.next_player_events[id].is_empty());
                //next_player_events.insert(*id, self.next_player_events[id].clone());
            }
        }
        self.next_player_events = next_player_events;
    }

    /// Allocates a new net entity id. Used by `entities::build_net`
    pub fn next_entity_id(&mut self) -> EntityId {
        self.entity_id_counter += 1;
        self.entity_id_counter
    }

    /// Queue an event only for one specific player
    pub fn add_player_event(&mut self, player_id: PlayerId, event: &GameEvent) {
        self.next_player_events.get_mut(&player_id).unwrap().push(event.clone());
    }

    pub fn entity_type_id(&self, type_name: &str) -> EntityTypeId {
        self.entity_types.iter()
            .enumerate()
            .find(|&(_, &(ref name, _))| name == &type_name)
            .unwrap()
            .0 as EntityTypeId
    }
}

impl ServiceManager for Services {}
