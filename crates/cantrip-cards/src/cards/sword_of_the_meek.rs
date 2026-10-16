use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects::ask_may;
use cantrip_oracle as oracle;

fn plus_one_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

/// "Whenever a 1/1 creature you control enters": it functions from its owner's graveyard (CR 113.6k), which is the one
/// zone its effect moves the card out of. The creature is read as it stands on the battlefield (CR 603.6a), counters and
/// static effects included; "that creature" is carried as the trigger's subject.
fn one_one_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { new, to, .. } = event else { return None };
    let sword = q.obj(source);
    if !to.is(ZoneKind::Battlefield) || !is_live(q, source) || sword.zone != Zone::graveyard(sword.owner) {
        return None;
    }
    let entered = q.obj(*new);
    if !is_live(q, *new) || !q.is_on_battlefield(*new) || entered.controller != sword.owner {
        return None;
    }
    let c = q.chars(*new);
    (c.is(CardType::Creature) && c.power == Some(1) && c.toughness == Some(1)).then_some(Fired { subject: Target::Object(*new), data: [0; 2] })
}

/// "You may return this card from your graveyard to the battlefield, then attach it to that creature." Nothing is asked
/// if the card is no longer the one that triggered (CR 400.7). It is attached only if that creature is still on the
/// battlefield and still a creature (CR 301.5c, 701.3b); otherwise it stays unattached.
fn return_and_attach(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let sword = item.object;
    if !is_live(g, sword) || g.obj(sword).zone != Zone::graveyard(item.controller) {
        return Ok(());
    }
    if !ask_may(g, item.controller, sword, Purpose::Optional)? {
        return Ok(());
    }
    let returned = g.put_onto_battlefield(sword, item.controller, false)?;
    if !is_live(g, returned) || !g.is_on_battlefield(returned) {
        return Ok(());
    }
    if let Target::Object(creature) = item.subject
        && is_live(g, creature)
        && g.is_on_battlefield(creature)
        && g.chars(creature).is(CardType::Creature)
    {
        g.attach(returned, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        equip(mana(2, 0, 0, 0, 0, 0), "Equip {2}"),
        triggered(when(EventMask::ZONE_CHANGE, one_one_enters), return_and_attach, "Whenever a 1/1 creature you control enters, you may return this card from your graveyard to the battlefield, then attach it to that creature.")
            .anywhere(),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +1/+2.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_one_plus_two,
        ..StaticDef::BLANK
    }],
    ..oracle::SWORD_OF_THE_MEEK
};
