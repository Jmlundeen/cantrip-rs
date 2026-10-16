use cantrip_api::*;
use cantrip_mechanics::history::activated_an_equip_ability_this_turn;
use cantrip_mechanics::{effects, storied};
use cantrip_oracle as oracle;

/// "As long as you have an enduring story, you may pay {0} rather than pay the equip cost of the first equip ability you
/// activate each turn" (CR 702.195b, 118.9, 602.2b): offered for an equip ability its controller activates while they have
/// the designation, if they have activated no equip ability yet this turn, on anyone's turn.
fn free_first_equip(q: &dyn Query, asker: &Asker, activator: PlayerId, _: ObjectId, def: &ActivatedDef) -> Option<ManaCost> {
    if !def.equip
        || activator != asker.controller
        || !storied::you_have_an_enduring_story(q.state(), asker.source)
        || activated_an_equip_ability_this_turn(q, activator)
    {
        return None;
    }
    Some(ManaCost::ZERO)
}

/// "Whenever another Dwarf or Equipment you control enters" (CR 603.6a): a permanent other than this one, under your
/// control as it entered, that is a Dwarf or an Equipment. "Dwarf" is a word of its text (CR 612); "Equipment" is no
/// creature type and is never changed.
fn another_dwarf_or_equipment_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && *new != me.source && q.controller(*new) == me.controller && {
                let c = q.chars(*new);
                c.has_subtype(q.words(me.source).subtype(Subtype::Dwarf)) || c.has_subtype(Subtype::Equipment)
            } =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    storied: true,
    ability_alternatives: &[free_first_equip],
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, another_dwarf_or_equipment_enters), |g, item| effects::draw(g, item, 1), "Whenever another Dwarf or Equipment you control enters, draw a card. This ability triggers only once each turn.")
        .triggers_each_turn(1)],
    ..oracle::K_LI_THE_RESOURCEFUL
};
