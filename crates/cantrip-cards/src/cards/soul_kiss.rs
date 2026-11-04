use cantrip_api::*;
use cantrip_mechanics::{effects, history, targets};
use cantrip_oracle as oracle;

/// "Activate no more than three times each turn" (CR 602.5b, 602.5c).
fn fewer_than_three_activations(q: &dyn Query, asker: &Asker) -> bool {
    history::this_ability_activations_this_turn(q, asker) < 3
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{B}").and(life(1)),
            effects::attached_creature_gets::<2, 2>,
            "{B}, Pay 1 life: Enchanted creature gets +2/+2 until end of turn. Activate no more than three times each turn.",
        )
        .condition(fewer_than_three_activations),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::SOUL_KISS
};
