use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip, may_attach_that_equipment};
use cantrip_mechanics::{statics, targets};
use cantrip_oracle as oracle;

/// "Whenever Hammer of Nazahn or another Equipment you control enters" (CR 603.6a): one ability over both entries. The
/// entering permanent is "that Equipment", fixed as the ability triggers.
fn equipment_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && (*new == source || q.chars(*new).has_subtype(Subtype::Equipment)) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, equipment_enters), may_attach_that_equipment, "Whenever Hammer of Nazahn or another Equipment you control enters, you may attach that Equipment to target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{4}"), "Equip {4}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 0>, "Equipped creature gets +2/+0."),
        static_ability(Layer::Ability, attached_object, statics::have_indestructible, "Equipped creature has indestructible."),
    ],
    ..oracle::HAMMER_OF_NAZAHN
};
