use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a land you control enters" carrying the entering land as the subject, so the resolve can ask whether it is a
/// Mountain (CR 603.6a).
fn a_land_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Land) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Koth deals 1 damage to each opponent. If that land is a Mountain, add {R}." The damage is dealt whether or not the
/// land has left; its last known characteristics determine whether it was a Mountain (CR 608.2h), the word read through
/// Koth's own words (CR 612).
fn landfall(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    if let Target::Object(land) = item.subject
        && g.chars(land).has_subtype(g.words(item.object).subtype(Subtype::Mountain))
    {
        g.add_mana(item.controller, Mana::R, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, a_land_you_control_enters),
        landfall,
        "Landfall \u{2014} Whenever a land you control enters, Koth deals 1 damage to each opponent. If that land is a Mountain, add {R}.",
    )],
    ..oracle::KOTH_THE_GEOMANCER
};
