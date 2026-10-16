use cantrip_api::*;
use cantrip_mechanics::attach::living_weapon;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;


fn keywords(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    for k in [Keyword::FirstStrike, Keyword::Trample, Keyword::Indestructible, Keyword::Haste] {
        chars.keywords = chars.keywords.with(k);
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
        equip(mana("{7}"), "Equip {7}"),
    ],
    delayed: &[triggered(when(EventMask::DAMAGE, deals_combat_damage_to_creature), exile_damaged, "Whenever this creature deals combat damage to a creature, exile that creature.")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<5, 5>, "Equipped creature gets +5/+5."),
        static_ability(Layer::Ability, attached_object, keywords, "Equipped creature has first strike, trample, indestructible and haste."),
        static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Equipped creature has \"Whenever this creature deals combat damage to a creature, exile that creature.\""),
    ],
    ..oracle::KALDRA_COMPLEAT
};


pub static EXTRA: &[&CardDef] = &[&cantrip_mechanics::tokens::GERM];
