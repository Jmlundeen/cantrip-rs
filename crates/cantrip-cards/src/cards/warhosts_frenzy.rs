use cantrip_api::*;
use cantrip_oracle as oracle;

/// Creatures you control get +2/+0 until end of turn: the creatures there are as it resolves (CR 611.2c). If this
/// spell was kicked, whenever a creature you control dies this turn, draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for id in g.permanents_of(item.controller) {
        if g.chars(id).is(CardType::Creature) {
            g.add_effect(item.object, id, EffectKind::Pump(2, 0), Duration::EndOfTurn);
        }
    }
    if g.obj(item.object).kicked > 0 {
        let this = g.card_id(&CARD);
        g.delayed_trigger(DelayedWhen::ThisTurn, item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    kicker: Some(Kicker { cost: mana(0, 0, 0, 1, 0, 0), multi: false, waterbend: false }),
    delayed: &[triggered(cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_DIES, cantrip_mechanics::effects::draw::<1>, "Whenever a creature you control dies this turn, draw a card.")],
    ..oracle::WARHOSTS_FRENZY
};
