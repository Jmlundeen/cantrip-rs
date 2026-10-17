use cantrip_api::*;
use cantrip_mechanics::effects::damage_that_player;
use cantrip_mechanics::triggers::{activation, ward_discard, OPPONENTS_CREATURE_ENTERS};
use cantrip_oracle as oracle;

/// "Whenever an opponent activates a loyalty ability" (CR 606.2, 603.2): a loyalty ability of any permanent, activated by a player
/// other than this ability's controller. The ability is about the player who activated it.
fn opponent_activates_a_loyalty_ability(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let (_, player, def) = activation(q, event)?;
    if def.is_loyalty_ability() && player != me.controller { Fired::about(Target::Player(player)) } else { None }
}

/// "Gideon deals 1 damage to that player."
fn one_damage_to_that_player(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_that_player(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_discard("Ward\u{2014}Discard a card."),
        triggered(OPPONENTS_CREATURE_ENTERS, one_damage_to_that_player, "Whenever a creature an opponent controls enters, Gideon deals 1 damage to that player."),
        triggered(when_asked(EventMask::ABILITY_ACTIVATED, opponent_activates_a_loyalty_ability), one_damage_to_that_player, "Whenever an opponent activates a loyalty ability, Gideon deals 1 damage to that player."),
    ],
    ..oracle::GIDEON_THE_OATHLESS
};
