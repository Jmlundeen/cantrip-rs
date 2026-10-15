use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Abilities you activate that aren't mana abilities cost {2} less to activate. This effect can't reduce the mana in
/// that cost to less than one mana" (CR 601.2f, 118.7): the core asks only non-mana abilities, whatever the zone they
/// are activated from.
fn two_less(q: &dyn Query, source: ObjectId, player: PlayerId, _: ObjectId, _: AbilityRef) -> u8 {
    if player == q.controller(source) { 2 } else { 0 }
}

/// {1}, {T}: Target creature can't block this turn.
fn cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::CantBlock, Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{1}, {T}: Target creature can't block this turn.",
        cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
        targets: &[targets::CREATURE],
        effect: cant_block,
        ..ActivatedDef::BLANK
    })],
    ability_discounts: &[two_less],
    ..oracle::ZIRDA_THE_DAWNWAKER
};
