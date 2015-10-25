use ecs::{EntityData, DataHelper};

use shared::GameEvent;
use shared::services::HasEvents;
use shared::movement::{WallInteractionType, WallInteraction};

use components::Components;
use services::Services;
use entities;

/// Bouncy enemy interaction with wall
pub struct BouncyEnemyWallInteraction;
impl WallInteraction<Components, Services> for BouncyEnemyWallInteraction {
    fn apply(&self,
             _enemy: EntityData<Components>, _wall: EntityData<Components>,
             _data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        WallInteractionType::Flip
    }
}

/// Projectile interaction with wall
pub struct ProjectileWallInteraction;
impl WallInteraction<Components, Services> for ProjectileWallInteraction {
    fn apply(&self,
             projectile: EntityData<Components>, _wall: EntityData<Components>,
             data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        let event = &GameEvent::ProjectileImpact {
            position: data.position[projectile].p,
        };
        data.services.add_event(&event);
        entities::remove_net(**projectile, data);
        WallInteractionType::Stop
    }
}