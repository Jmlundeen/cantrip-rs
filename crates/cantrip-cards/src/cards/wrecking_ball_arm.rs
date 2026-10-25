use cantrip_api::*;
use cantrip_mechanics::statics::base_pt;
use cantrip_mechanics::attach::{attached_object, equip, equip_quality};
use cantrip_oracle as oracle;

/// The equipped creature: what the Equipment is attached to while it is still that creature (CR 301.5a).
fn equipped_creature(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    attached_object(view, source, affected, so_far) && view.chars(affected).is(CardType::Creature)
}

/// "Equipped creature ... can't be blocked by creatures with power 2 or less" (CR 509.1b): no "has", so a rule of the
/// Equipment's, not an ability the creature gains (CR 113.1a); a creature that loses all its abilities is still under it
/// (CR 613.1f). A blocker with no power can't block it either, as with the keyword.
fn evasion(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    q.obj(source).attached_to != Some(attacker) || q.chars(blocker).power.is_some_and(|p| p > 2)
}

fn legendary_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Creature) && c.supertypes.legendary
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        equip_quality(mana("{3}"), &[target(legendary_creature_you_control)], "Equip legendary creature {3}"),
        equip(mana("{7}"), "Equip {7}"),
    ],
    statics: &[
        static_ability(Layer::PtSet, equipped_creature, base_pt::<7, 7>, "Equipped creature has base power and toughness 7/7."),
    ],
    block_rules: &[evasion],
    ..oracle::WRECKING_BALL_ARM
};
