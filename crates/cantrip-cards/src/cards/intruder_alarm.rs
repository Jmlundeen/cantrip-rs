use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures don't untap during their controllers' untap steps" (CR 502.3): every creature, whoever controls it,
/// is read as the untap step untaps. It is a rule of the Alarm, never an ability of the creatures, so it is applied
/// in the last layer, after anything that took their abilities away (CR 613.1f), and is the "does not untap" the
/// untap step already asks every permanent for.
fn creatures(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature)
}

fn does_not_untap(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::DoesNotUntap);
}

/// "Whenever a creature enters": any creature under any player's control, and the Alarm itself if it enters as one.
fn creature_enters(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield) && q.chars(*new).is(CardType::Creature) => Fired::PLAIN,
        _ => None,
    }
}

/// Untap all creatures: every creature on the battlefield, all players'.
fn untap_all(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        g.untap(id);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, creature_enters), untap_all, "Whenever a creature enters, untap all creatures.")],
    statics: &[StaticDef {
        text: "Creatures don't untap during their controllers' untap steps.",
        layer: Layer::PtModify,
        affects: creatures,
        apply: does_not_untap,
        ..StaticDef::BLANK
    }],
    ..oracle::INTRUDER_ALARM
};
