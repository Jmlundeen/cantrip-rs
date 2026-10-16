use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever a player activates an ability that isn't a mana ability" (CR 602.2, 603.2): any player's activation of any
/// object's ability, wherever it functions (a cycling from a hand, a loyalty ability), this creature's own included. The
/// ability is about that player.
fn player_activates_non_mana_ability(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    let (_, player, def) = triggers::activation(q, event)?;
    if def.is_mana_ability() { None } else { Fired::about(Target::Player(player)) }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ABILITY_ACTIVATED, player_activates_non_mana_ability),
        |g, item| effects::damage_that_player(g, item, 1),
        "Whenever a player activates an ability that isn't a mana ability, this creature deals 1 damage to that player.",
    )],
    ..oracle::BURNING_TREE_SHAMAN
};
