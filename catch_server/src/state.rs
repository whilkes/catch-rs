use std::f32;
use std::collections::HashMap;

use ecs;
use rand;
use hprof;
use na::{Vec2, Norm};

use shared::{NEUTRAL_PLAYER_ID, TickNumber, GameInfo, DeathReason, GameEvent, PlayerId, PlayerInfo,
             Item};
use shared::services::HasEvents;
use shared::map::Map;
use shared::net::TimedPlayerInput;

use components::WallPosition;
use systems::Systems;
use services::Services;
use entities;

const RESPAWN_TIME_S: f32 = 5.0;

pub struct Player {
    // Has this player been sent its first tick yet?
    is_new: bool,

    // If true, player (and owned entities) will be removed next tick
    remove: bool,

    info: PlayerInfo,

    // Entity controlled by the player, if alive
    entity: Option<ecs::Entity>,

    respawn_time: Option<f32>, 
}

pub struct SpawnPoint {
    position: Vec2<f32>,
    size: Vec2<f32>,
}

impl Player {
    fn new(info: PlayerInfo) -> Player {
        Player {
            is_new: true,
            remove: false,
            info: info,
            entity: None,
            respawn_time: Some(0.0),
        }
    }

    fn alive(&self) -> bool {
        self.entity.is_some()
    }
}

pub struct GameState {
    game_info: GameInfo,
    map: Map,
    spawn_points: Vec<SpawnPoint>,
    pub world: ecs::World<Systems>, 
    pub tick_number: TickNumber,
    time_s: f32,
    players: HashMap<PlayerId, Player>,
}

impl GameState {
    pub fn new(game_info: &GameInfo) -> GameState {
        let map = Map::load(&game_info.map_name).unwrap();

        let spawn_points = map.objects.iter()
               .filter(|object| &object.type_str == "player_spawn")
               .map(|object| SpawnPoint {
                        position: Vec2::new(object.x, object.y),
                        size: Vec2::new(object.width, object.height),
                    })
               .collect();

        let services = Services::new(game_info.entity_types.clone());

        GameState {
            game_info: game_info.clone(),
            map: map,
            spawn_points: spawn_points,
            world: ecs::World::with_services(services),
            tick_number: 0,
            time_s: 0.0,
            players: HashMap::new(),
        }
    }

    fn create_map_objects(&mut self) {
        for object in self.map.objects.iter() {
            if &object.type_str == "item_spawn" {
                let entity = entities::build_net(&object.type_str, 0, &mut self.world.data);
                self.world.with_entity_data(&entity, |e, c| {
                    c.position[e].p = Vec2::new(object.x, object.y);
                });
            } else if &object.type_str == "bouncy_enemy" {
                let entity = entities::build_net(&object.type_str, 0, &mut self.world.data);
                self.world.with_entity_data(&entity, |e, c| {
                    c.position[e].p = Vec2::new(object.x, object.y);
                    c.orientation[e].angle = rand::random::<f32>() * f32::consts::PI * 2.0;
                    c.bouncy_enemy[e].attract = rand::random::<bool>();
                });
            } else if &object.type_str == "player_spawn" {
            } else {
                warn!("ignoring unknown entity type {} in map", object.type_str);
            }
        }

        for &(pos_a, pos_b) in self.map.lines.iter() {
            let entity = entities::build_net("wall_wood", 0, &mut self.world.data);
            self.world.with_entity_data(&entity, |e, c| {
                c.wall_position[e] = WallPosition {
                    pos_a: pos_a,
                    pos_b: pos_b
                };
            });
        }
    }

    // For adding test entities and stuff
    fn init_first_tick(&mut self) {
        self.create_map_objects();

        /*let num_bouncies = 50;

        for _ in 0..num_bouncies {
            let entity = entities::build_net("bouncy_enemy", 0, &mut self.world.data);

            // Pick a random non-blocked tile
            let mut rx;
            let mut ry;
            loop {
                rx = rand::random::<usize>() % self.map.width();
                ry = rand::random::<usize>() % self.map.height();

                if self.map.get_tile(LayerId::Block, rx, ry).is_none() {
                    break;
                }
            }

            let position = [(rx * self.map.tile_width()) as f64 +
                            self.map.tile_width() as f64 / 2.0,
                            (ry * self.map.tile_height()) as f64 +
                            self.map.tile_height() as f64 / 2.0];

            self.world.with_entity_data(&entity, |e, c| {
                c.position[e].p = position;
                c.orientation[e].angle = rand::random::<f64>() * f64::consts::PI * 2.0;
            });
        }*/
        
        let num_walls = 30;
        let width = self.map.width_pixels() as f32;
        let height = self.map.height_pixels() as f32;

        /*for _ in 0..num_walls {
            let entity = entities::build_net("wall_wood", 0, &mut self.world.data);

            let ax = rand::random::<f32>() * width;
            let ay = rand::random::<f32>() * height;
            let phi = rand::random::<f32>() * f32::consts::PI * 2.0;
            let r = rand::random::<f32>() * (400.0 - 32.0) + 32.0;
            let bx = ax + phi.cos() * r;
            let by = ay + phi.sin() * r;

            //println!("wall at {},{},{},{}", ax, ay, bx, by);

            self.world.with_entity_data(&entity, |e, c| {
                c.wall_position[e] = WallPosition {
                    pos_a: Vec2::new(ax, ay),
                    pos_b: Vec2::new(bx, by), 
                };
            });
        }*/

        /*let entity = entities::build_net("wall_wood", 0, &mut self.world.data);
        self.world.with_entity_data(&entity, |e, c| {
            c.wall_position[e] = WallPosition {
                pos_a: Vec2::new(0.0, 0.0),
                pos_b: Vec2::new(width, 0.0)
            };
        });
        let entity = entities::build_net("wall_wood", 0, &mut self.world.data);
        self.world.with_entity_data(&entity, |e, c| {
            c.wall_position[e] = WallPosition {
                pos_a: Vec2::new(0.0, 0.0),
                pos_b: Vec2::new(0.0, height)
            };
        });
        let entity = entities::build_net("wall_wood", 0, &mut self.world.data);
        self.world.with_entity_data(&entity, |e, c| {
            c.wall_position[e] = WallPosition {
                pos_a: Vec2::new(width, 0.0),
                pos_b: Vec2::new(width, height)
            };
        });
        let entity = entities::build_net("wall_wood", 0, &mut self.world.data);
        self.world.with_entity_data(&entity, |e, c| {
            c.wall_position[e] = WallPosition {
                pos_a: Vec2::new(0.0, height), 
                pos_b: Vec2::new(width, height)
            };
        });*/

        self.world.flush_queue();
    }

    pub fn tick_number(&self) -> TickNumber {
        self.tick_number 
    }

    pub fn add_player(&mut self, id: PlayerId, info: PlayerInfo) {
        assert!(self.players.get(&id).is_none());
        self.players.insert(id, Player::new(info));
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        // The player will be removed at the start of the next tick
        self.players.get_mut(&id).unwrap().remove = true;
    }

    fn spawn_player(&mut self, id: PlayerId) -> ecs::Entity {
        assert!(self.players[&id].entity.is_none(),
                "Can't spawn a player that is already controlling an entity");

        let entity = entities::build_net("player", id, &mut self.world.data);

        self.players.get_mut(&id).unwrap().entity = Some(entity);

        // Pick a random spawn point
        let position = {
            let spawn_point = &self.spawn_points[rand::random::<usize>() %
                                                 self.spawn_points.len()];
            Vec2::new(spawn_point.position[0] + rand::random::<f32>() * spawn_point.size[0],
                      spawn_point.position[1] + rand::random::<f32>() * spawn_point.size[1])
        };

        // If we don't have a catcher right now, this player is lucky
        let is_catcher = self.current_catcher() == None; 

        self.world.with_entity_data(&entity, |e, c| {
            c.position[e].p = position;
            c.player_state[e].invulnerable_s = Some(2.5);
            c.player_state[e].is_catcher = is_catcher;
            c.player_state[e].has_shield = true;

            // We'll equip a gun for now
            //c.player_state[e].equip(0, Item::Weapon { charges: 20 }); 
            //c.player_state[e].equip(1, Item::FragWeapon { charges: 2 });
            //c.player_state[e].equip(2, Item::BallSpawner { charges: 3 }); 
        });

        entity
    }

    pub fn get_player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.players[&id].info
    }

    pub fn on_player_input(&mut self,
                           id: PlayerId,
                           input: &TimedPlayerInput) {
        if let Some(entity) = self.players.get_mut(&id).unwrap().entity {
            self.world.data.with_entity_data(&entity, |player, c| {
                c.player_controller[player].inputs.push(input.clone()); 
            });
        }
    }

    fn current_catcher(&mut self) -> Option<PlayerId> {
        for (player_id, player) in self.players.iter() {
            if let Some(entity) = player.entity {
                if self.world.with_entity_data(&entity, |e, c| c.player_state[e].is_catcher)
                       .unwrap() {
                    return Some(*player_id);
                }
            }
        }
        return None;
    }

    /// Advances the state of the server by one tick.
    /// Events generated during the tick are stored for each player separately in the services.
    pub fn tick(&mut self) {
        self.check_integrity();

        self.tick_number += 1;
        self.world.services.tick_dur_s = 1.0 / (self.game_info.ticks_per_second as f32);

        // The order of the following operations is important, in order to avoid sending
        // invalid or duplicate events to players.
        // Note that game events should only be created in the scope of this tick function.
        
        // Initialize the event queue of each player to be empty
        self.world.services.prepare_for_tick(self.tick_number, self.players.keys().map(|i| *i));

        // First, handle adding new players. Send out events to new players to replicate our state
        // and entities. This means queueing up InitialPlayerList and CreateEntity events.
        // We also send out PlayerJoin events to non-new players.
        self.tick_add_new_players();
        
        // Remove any players that disconnected this tick.
        // This means broadcasting PlayerLeave and RemoveEntity events.
        self.tick_remove_disconnected_players();

        // Send out a table of player stats (we probably don't need to do this every frame)
        self.tick_replicate_player_stats();

        // Create some initial entities, e.g. from the map specified in self.game_info
        if self.tick_number == 1 {
            self.init_first_tick();
        }

        // Check if we can respawn some players
        self.tick_respawn_players();

        // Finally, run the input queued up for every player (via ClientMessage::PlayerInput).
        // This is the only place where the time of player-controlled entities is advanced.
        // If we don't receive any input from a player, their entity does not move.
        self.tick_run_player_input();

        // Let all the systems know about any new or removed ecs entities
        self.world.flush_queue();
 
        // Advance the state of server-controlled entities
        {
            let _g = hprof::enter("entities");

            self.world.systems.movement_system.tick(&mut self.world.data);
            self.world.systems.bouncy_enemy_system.tick(&mut self.world.data);
            self.world.systems.projectile_system.tick(&mut self.world.data);
            self.world.systems.item_spawn_system.tick(&mut self.world.data);
            self.world.systems.rotate_system.tick(&mut self.world.data);
            self.world.systems.interaction_system.tick(&mut self.world.data);
        }
        
        // Process events generated in this tick
        for i in 0..self.world.services.next_events.len() {
            let event = self.world.services.next_events[i].clone();
            self.tick_process_event(event);
        }
        self.world.services.next_events.clear();

        self.world.flush_queue();

        self.time_s += self.world.services.tick_dur_s;
    }

    fn tick_add_new_players(&mut self) {
        // Find new and non-new players
        let mut new_players = vec![];
        let mut non_new_players = vec![];
        for (&player_id, player) in self.players.iter_mut() {
            if player.is_new {
                new_players.push(player_id);
                player.is_new = false;
            } else {
                non_new_players.push(player_id);
            }
        }

        // Replicate the game state to the new clients
        for new_player_id in new_players {
            info!("replicating net state to player {}", new_player_id);

            // First, tell them about the player list (note: this already includes themselves!)
            let players = self.players.iter().map(|(k, v)| (*k, v.info.clone())).collect();
            let event = GameEvent::InitialPlayerList(players);
            self.world.services.add_player_event(new_player_id, &event);

            // Now we can create entities
            self.world.systems.net_entity_system
                .replicate_entities(new_player_id, &mut self.world.data);

            // Tell any non-new players about this new player
            let new_player_info = self.players[&new_player_id].info.clone();
            let event = GameEvent::PlayerJoin(new_player_id, new_player_info);
            for &player_id in &non_new_players {
                self.world.services.add_player_event(player_id, &event);
            }
        }
    }

    fn tick_remove_disconnected_players(&mut self) {
        let mut remove = Vec::new();
        for (&player_id, player) in self.players.iter_mut() {
            if player.remove {
                info!("removing player {}", player_id);
                remove.push(player_id);
            }
        }

        for &id in remove.iter() {
            // Was this player the catcher?
            let is_catcher = if let Some(entity) = self.players[&id].entity {
                self.world.with_entity_data(&entity, |e, c| {
                    c.player_state[e].is_catcher
                }).unwrap()
            } else {
                false
            };

            self.world.systems.net_entity_system.remove_player_entities(id, &mut self.world.data);
            self.players.remove(&id); 
            self.world.services.add_event(&GameEvent::PlayerLeave(id));

            // If the disconnected player was the catcher, choose a random new alive one as catcher
            let alive_players = self.players.iter()
                                    .filter(|&(_, player)| player.alive())
                                    .map(|(&id, _)| id)
                                    .collect::<Vec<_>>();
            if is_catcher && !alive_players.is_empty() {
                let chosen_one = alive_players[rand::random::<usize>() % alive_players.len()];

                self.world.with_entity_data(&self.players[&chosen_one].entity.unwrap(), |e, c| {
                    assert!(!c.player_state[e].is_catcher);
                    c.player_state[e].is_catcher = true;
                });
            }
        }

        // Allow systems to remove references to newly-removed entities
        self.world.flush_queue();
    }

    fn tick_replicate_player_stats(&mut self) {
        let stats = self.players.iter().map(|(&id, p)| (id, p.info.stats.clone())).collect();
        let event = GameEvent::UpdatePlayerStats(stats);
        self.world.services.add_event(&event);
    }

    fn tick_respawn_players(&mut self) {
        let mut respawn = Vec::new();
        for (&player_id, player) in self.players.iter_mut() {
            if !player.alive() {
                if let Some(time) = player.respawn_time {
                    let time = time - self.world.services.tick_dur_s;

                    player.respawn_time = if time <= 0.0 {
                        respawn.push(player_id);
                        None
                    } else {
                        Some(time)
                    };
                }

            }
        }

        for player_id in respawn {
            self.spawn_player(player_id); 
        }

        // Allow systems to add references to new entities
        self.world.flush_queue();
    }

    fn tick_run_player_input(&mut self) {
        self.world.systems.player_controller_system
            .run_queued_inputs(&mut self.world.data);
    }

    fn tick_process_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::PlayerDied {
                player_id,
                position,
                responsible_player_id,
                reason,
            } => {
                self.on_player_died(player_id, position, responsible_player_id, reason);
            }
            _ => ()
        }
    }

    fn on_player_died(&mut self, player_id: PlayerId, position: Vec2<f32>,
                      responsible_player_id: PlayerId, reason: DeathReason) {
        info!("killing player {}", player_id);

        assert!(self.players.get(&player_id).is_some());
        assert!(responsible_player_id == NEUTRAL_PLAYER_ID ||
                self.players.get(&responsible_player_id).is_some(),
                "disconnected players shouldn't be able to kill (yet?)");

        if !self.players[&player_id].alive() {
            debug!("killing a dead player! hah!");
        } else {
            let player_entity = self.players[&player_id].entity.unwrap();

            // Update the score
            {
                let player = self.players.get_mut(&player_id).unwrap();
                player.info.stats.deaths += 1;
            }
            if responsible_player_id != NEUTRAL_PLAYER_ID {
                let responsible_player = self.players.get_mut(&responsible_player_id).unwrap();
                responsible_player.info.stats.score +=
                    match reason {
                        DeathReason::Caught => 10,
                        _ => 1,
                    };
            }

            // If this player is the catcher, we need to determine a new catcher
            let is_catcher = self.world.with_entity_data(&player_entity, |e, c| {
                let is_catcher = c.player_state[e].is_catcher;
                c.player_state[e].is_catcher = false;
                is_catcher
            }).unwrap();

            if is_catcher {
                let responsible_entity = 
                    if responsible_player_id != player_id {
                        self.players.get(&responsible_player_id).map(|player| player.entity)
                    } else {
                        None
                    };

                if let Some(Some(responsible_entity)) = responsible_entity {
                    // If we were killed by another player, that one becomes the catcher
                    self.world.with_entity_data(&responsible_entity, |e, c| {
                        c.player_state[e].is_catcher = true;
                    });
                } else {
                    // Otherwise, find the player that is the closest to the dead catcher
                    let player_ids = self.players.keys().filter(|id| **id != player_id);

                    let mut closest: Option<(ecs::Entity, f32)> = None; 
                    for &id in player_ids {
                        if let Some(entity) = self.players[&id].entity {
                            let d = self.world.with_entity_data(&entity, |e, c| {
                                (position - c.position[e].p).norm()
                            }).unwrap();
                            if closest.is_none() || closest.unwrap().1 > d {
                                closest = Some((entity, d));
                            }
                        }
                    }
                    
                    if let Some((closest_player_entity, _)) = closest {
                        self.world.with_entity_data(&closest_player_entity, |e, c| {
                            assert!(!c.player_state[e].is_catcher);
                            c.player_state[e].is_catcher = true;
                        });
                    } else {
                        // If we are here, this should mean that nobody is alive
                        for (id, player) in self.players.iter() {
                            assert!(*id == player_id || player.entity.is_none());
                        }
                    }
                }
            }

            // Kill the player
            {
                let player = self.players.get_mut(&player_id).unwrap();
                player.entity = None;
                player.respawn_time = Some(RESPAWN_TIME_S);
            };

            entities::remove_net(player_entity, &mut self.world.data);
        }
    }

    fn check_integrity(&mut self) {
        // When we have at least one player that is alive, there should be exactly one catcher
        let mut num_alive = 0;
        let mut num_catchers = 0;

        for (_, player) in self.players.iter() {
            if let Some(entity) = player.entity {
                num_alive += 1;
                self.world.with_entity_data(&entity, |e, c| {
                    if c.player_state[e].is_catcher {
                        num_catchers += 1;
                    }
                });
            }
        }

        if num_alive > 0 {
            assert!(num_catchers == 1, "There should be exactly one catcher!");
        }
    }
}
