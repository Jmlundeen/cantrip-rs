use cantrip_api::*;
use cantrip_mechanics::attach::living_weapon;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;


fn plus_five(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 5);
    chars.toughness = chars.toughness.map(|v| v + 5);
}

fn keywords(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    for k in [Keyword::FirstStrike, Keyword::Trample, Keyword::Indestructible, Keyword::Haste] {
        chars.keywords = chars.keywords.with(k);
    }
}

/// The equipped creature has the Equipment's delayed ability 0 (CR 301.5a, 613.1f): a triggered ability it controls,
/// gone the moment the Equipment stops being attached to it.
fn grant_exile(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

/// "Whenever this creature deals combat damage to a creature": once for each creature damaged, judged as the damage
/// is dealt.
fn deals_combat_damage_to_creature(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Object(id), amount, combat: true }
            if *from == source && *amount > 0 && q.chars(*id).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*id))
        }
        _ => None,
    }
}

/// Exile that creature: nothing if it has already left the battlefield (CR 400.7).
fn exile_damaged(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject
        && cantrip_mechanics::targets::permanent(g, creature)
    {
        g.exile(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, living_weapon, "Living weapon (When this Equipment enters, create a 0/0 black Phyrexian Germ creature token, then attach this to it.)"),
        equip(mana(7, 0, 0, 0, 0, 0), "Equip {7}"),
    ],
    delayed: &[triggered(when(EventMask::DAMAGE, deals_combat_damage_to_creature), exile_damaged, "Whenever this creature deals combat damage to a creature, exile that creature.")],
    statics: &[
        StaticDef {
            text: "Equipped creature gets +5/+5.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_five,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has first strike, trample, indestructible and haste.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: keywords,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has \"Whenever this creature deals combat damage to a creature, exile that creature.\"",
            layer: Layer::Ability,
            affects: attached_object,
            apply: grant_exile,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::KALDRA_COMPLEAT
};


pub static EXTRA: &[&CardDef] = &[&cantrip_mechanics::tokens::GERM];
