use cantrip_api::*;
use cantrip_mechanics::storm;
use cantrip_mechanics::replacements::BUYBACK;
use cantrip_oracle as oracle;

/// Creatures you control get +1/+0 until end of turn: those there as it resolves (CR 611.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for id in g.permanents_of(item.controller) {
        if g.chars(id).is(CardType::Creature) {
            g.add_effect(item.object, id, EffectKind::Pump(1, 0), Duration::EndOfTurn);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    kicker: Some(Kicker { cost: mana(2, 0, 0, 0, 0, 0), multi: false, waterbend: false }),
    replacements: &[BUYBACK],
    abilities: &[storm::trigger("Storm (When you cast Haze of Rage, copy it for each spell cast before it this turn.)")],
    ..oracle::HAZE_OF_RAGE
};
