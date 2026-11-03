use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::EACH_END_STEP};
use cantrip_oracle as oracle;

/// The intervening condition is checked as the ability triggers and resolves (CR 603.4).
fn no_creatures(q: &dyn Query, _: &Asker) -> bool {
    !q.state().battlefield().iter().any(|&id| q.chars(id).is(CardType::Creature))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(EACH_END_STEP, effects::sacrifice_self, "At the beginning of the end step, if no creatures are on the battlefield, sacrifice this enchantment.")
            .condition(no_creatures),
        ability(cost("{B}"), effects::damage_each_creature_and_player::<1>, "{B}: This enchantment deals 1 damage to each creature and each player."),
    ],
    ..oracle::PESTILENCE
};
