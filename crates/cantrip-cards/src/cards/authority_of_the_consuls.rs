use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures your opponents control enter tapped." (CR 614.1c.) The creature is judged as it would exist on the
/// battlefield (CR 614.12), under the player it enters under.
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let Proposal::ZoneChange(z) = proposal else { return false };
    z.to.is(ZoneKind::Battlefield) && z.controller != asker.controller && q.chars(z.object).is(CardType::Creature)
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = true;
    }
}

/// "Whenever a creature an opponent controls enters" (CR 603.6a): any creature, token or not, under the control of
/// a player other than this enchantment's controller as it entered.
fn opponents_creature_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) != q.controller(source) && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// You gain 1 life.
fn gain_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "enters tapped")],
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, opponents_creature_enters), gain_1, "Whenever a creature an opponent controls enters, you gain 1 life.")],
    ..oracle::AUTHORITY_OF_THE_CONSULS
};
