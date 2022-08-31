use crate::prelude::*;

pub mod audio;
pub mod inventory;
pub mod movement;
pub mod prelude;
pub mod weapon;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        use prelude::*;

        app.add_startup_system(Player::label_system)
            .add_startup_system(PlayerInteractVolume::label_system)
            .add_exit_system(GameState::GameOver, Player::restart_system)
            .add_plugin(movement::PlayerMovementPlugin)
            .add_plugin(audio::PlayerAudioPlugin)
            .add_plugin(weapon::PlayerWeaponPlugin)
            .add_plugin(inventory::PlayerInventoryPlugin);
    }
}
