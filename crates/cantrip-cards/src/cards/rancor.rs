use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "When this Aura is put into a graveyard from the battlefield" (CR 700.4), looked back on (CR 603.10a). "It" is the
/// card in the graveyard, a new object (CR 400.7), the ability's subject.
fn put_into_graveyard(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. } if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// Return it to its owner's hand, provided it is still in the graveyard.
fn return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(card) = item.subject {
        let o = g.obj(card);
        if o.became.is_none() && o.zone == Zone::graveyard(o.owner) {
            g.return_to_hand(&[card]);
        }
    }
    Ok(())
}

fn plus_two_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
}

fn trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, put_into_graveyard), return_to_hand, "When Rancor is put into a graveyard from the battlefield, return it to its owner's hand.")
        .look_back()],
    statics: &[
        StaticDef {
            text: "Enchanted creature gets +2/+0.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_two_trample,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Enchanted creature has trample.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: trample,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::RANCOR
};
