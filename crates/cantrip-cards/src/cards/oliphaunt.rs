use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

fn another_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t) && t != Target::Object(asker.source)
}

/// "Another target creature you control gets +2/+0 and gains trample until end of turn."
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, 2, 0);
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ATTACKS, rally, "Whenever Oliphaunt attacks, another target creature you control gets +2/+0 and gains trample until end of turn.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: another_creature_you_control }]),
        typecycling::ability(Subtype::Mountain, mana(1, 0, 0, 0, 0, 0), "Mountaincycling {1} ({1}, Discard this card: Search your library for a Mountain card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::OLIPHAUNT
};
