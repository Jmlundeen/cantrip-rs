use cantrip_api::*;
use cantrip_mechanics::{effects, history, triggers::EACH_END_STEP};
use cantrip_oracle as oracle;

/// The intervening condition is checked at triggering and resolution (CR 603.4).
fn no_creatures(q: &dyn Query, _: &Asker) -> bool {
    !q.state().battlefield().iter().any(|&id| q.chars(id).is(CardType::Creature))
}

/// Deal the damage simultaneously (CR 120.3).
fn damage_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_creature_and_player_n(g, item, 1);
    Ok(())
}

/// "Activate no more times each turn than the number of snow Swamps you control" (CR 602.5b, 602.5c), the subtype read through the
/// words of the source's text (CR 612).
fn fewer_activations_than_snow_swamps(q: &dyn Query, asker: &Asker) -> bool {
    let swamp = q.words(asker.source).subtype(Subtype::Swamp);
    let snow_swamps = q.state().battlefield().iter()
        .filter(|&&id| q.controller(id) == asker.controller && q.chars(id).supertypes.snow && q.chars(id).has_subtype(swamp))
        .count();
    history::this_ability_activations_this_turn(q, asker) < snow_swamps
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
                EACH_END_STEP,
                effects::sacrifice_self,
                "At the beginning of the end step, if no creatures are on the battlefield, sacrifice this enchantment.",
            )
            .condition(no_creatures),
        ability(
                cost("{B}"),
                damage_each,
                "{B}: This enchantment deals 1 damage to each creature and each player. Activate no more times each turn than the number of snow Swamps you control.",
            )
            .condition(fewer_activations_than_snow_swamps),
    ],
    ..oracle::WITHERING_WISPS
};
