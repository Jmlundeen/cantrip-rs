use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Sacrifice a land: You gain 2 life." Not a mana ability: it uses the stack.
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice a land: You gain 2 life.",
        cost: Cost::sacrifice(1, cantrip_mechanics::targets::filters::land),
        effect: cantrip_mechanics::effects::gain_life::<2>,
        ..ActivatedDef::BLANK
    })],
    ..oracle::ZURAN_ORB
};
