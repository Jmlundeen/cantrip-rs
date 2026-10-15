use cantrip_api::*;
use cantrip_oracle as oracle;

/// Take an extra turn after this one (CR 500.7).
fn extra_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Sacrifice five artifacts: Take an extra turn after this one.",
        cost: Cost { tap: true, permanents: Some(PermanentCost { count: 5, filter: cantrip_mechanics::targets::filters::artifact, to_hand: false }), ..Cost::NONE },
        effect: extra_turn,
        ..ActivatedDef::BLANK
    })],
    ..oracle::TIME_SIEVE
};
