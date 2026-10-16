use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever Giott or another Dwarf you control enters and whenever an Equipment you control enters" (CR 603.6a): one ability
/// with two trigger events, both entering the battlefield under my controller's control. An object that enters is one event, so
/// an object that is both a Dwarf and an Equipment triggers it once (CR 603.2c). "Dwarf" is a creature type of Giott's own text,
/// read through its words (CR 612); Giott itself always counts, whatever the word says.
fn dwarf_or_equipment_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && (*new == me.source
                    || q.chars(*new).has_subtype(q.words(me.source).subtype(Subtype::Dwarf))
                    || q.chars(*new).has_subtype(Subtype::Equipment)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, dwarf_or_equipment_enters),
        |g, item| effects::may_discard_to_draw(g, item, 1),
        "Whenever Giott or another Dwarf you control enters and whenever an Equipment you control enters, you may discard a card. If you do, draw a card.",
    )],
    ..oracle::GIOTT_KING_OF_THE_DWARVES
};
