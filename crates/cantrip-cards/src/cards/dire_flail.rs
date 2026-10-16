use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{choose, craft, damage_target, targets};
use cantrip_oracle as oracle;

/// "Equipped creature gets +2/+0."
fn plus_two_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
}

/// "Equipped creature gets +3/+0."
fn plus_three_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 3);
}

/// How many Dire Blunderbusses' abilities one creature can tell apart: each Blunderbuss attached to it gives it an
/// instance of the attack ability of its own (CR 613.1f), and each instance names its own Blunderbuss as "Dire
/// Blunderbuss". More than this many on one creature would give it no more.
const INSTANCES: usize = 8;

/// The Dire Blunderbusses that give `creature` the attack ability, in the order their static abilities apply to it (CR
/// 613.7: the timestamp order of the Equipment): the nth instance on the creature is the nth of these.
fn blunderbusses_on(q: &dyn Query, creature: ObjectId) -> Vec<ObjectId> {
    let back = q.card_id(&BACK);
    let mut on: Vec<ObjectId> = q
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| q.obj(id).attached_to == Some(creature) && q.chars(id).card == back && !q.state().lost_all_abilities(id))
        .collect();
    on.sort_by_key(|&id| q.obj(id).timestamp);
    on
}

/// "... and has 'Whenever this creature attacks, ...'" (CR 613.1f): the equipped creature gets the next of the
/// Blunderbuss's attack abilities that it does not have yet, so a second Blunderbuss gives a second instance, which is its
/// own.
fn grant_attack_ability(view: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    let back = view.cards().id_of(&BACK);
    let has = chars.abilities.iter().filter(|a| a.is_some_and(|a| a.card == back && a.is_delayed())).count();
    if has < INSTANCES
        && let Some(slot) = chars.abilities.iter().position(|a| a.is_none())
    {
        chars.abilities[slot] = Some(AbilityRef::delayed(back, has as u8));
    }
}

/// "Whenever this creature attacks": the creature with the ability, declared as an attacker (CR 508.1). The instance
/// is the `K`th on the creature, so "Dire Blunderbuss" in it is the `K`th Blunderbuss attached to it, carried as the
/// ability's subject.
fn this_creature_attacks<const K: usize>(q: &dyn Query, creature: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == creature => {
            Fired::about(Target::Object(*blunderbusses_on(q, creature).get(K)?))
        }
        _ => None,
    }
}

/// "You may sacrifice an artifact other than Dire Blunderbuss. When you do, this creature deals damage equal to its power
/// to target creature." Declining is the empty answer to an up-to-one choice; the reflexive ability (CR 603.12) exists
/// only if an artifact was sacrificed, and its target is chosen as it goes on the stack (the card's ruling).
fn may_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let blunderbuss = item.subject.object();
    let artifacts: Vec<ObjectId> =
        g.permanents_of(you).into_iter().filter(|&id| Some(id) != blunderbuss && g.chars(id).is(CardType::Artifact)).collect();
    let chosen = choose::cards(g, you, item.object, Purpose::Sacrifice, artifacts, 1, true)?;
    let Some(&artifact) = chosen.first() else { return Ok(()) };
    // A permanent of your own is always yours to sacrifice to your own ability (CR 701.21a): chosen, it is sacrificed.
    g.sacrifice(&[artifact]);
    let back = g.card_id(&BACK);
    g.reflexive_trigger(item.object, you, AbilityRef::delayed(back, INSTANCES as u8), Target::None);
    Ok(())
}

/// "This creature deals damage equal to its power to target creature": its power as the ability resolves, or as it last
/// existed on the battlefield if it has left (the card's rulings, CR 608.2h); the creature is the source, so its lifelink
/// counts as it last was.
fn deal_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u16;
    damage_target(g, item, 0, power);
    Ok(())
}

const ATTACK_TEXT: &str = "Whenever this creature attacks, you may sacrifice an artifact other than Dire Blunderbuss. When you do, this creature deals damage equal to its power to target creature.";

pub static BACK: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    delayed: &[
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<0>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<1>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<2>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<3>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<4>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<5>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<6>), may_sacrifice, ATTACK_TEXT),
        triggered(when(EventMask::ATTACKS, this_creature_attacks::<7>), may_sacrifice, ATTACK_TEXT),
        delayed_trigger(deal_power, "When you do, this creature deals damage equal to its power to target creature.").targets(&[targets::CREATURE]),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_three_power, "Equipped creature gets +3/+0."),
        static_ability(Layer::Ability, attached_object, grant_attack_ability, "Equipped creature has \"Whenever this creature attacks, you may sacrifice an artifact other than Dire Blunderbuss. When you do, this creature deals damage equal to its power to target creature.\""),
    ],
    ..oracle::DIRE_BLUNDERBUSS
};

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}"), craft::ability(mana("{3}{R}{R}"), craft::ARTIFACT, "Craft with artifact {3}{R}{R}")],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_two_power, "Equipped creature gets +2/+0.")],
    back: Some(&BACK),
    ..oracle::DIRE_FLAIL
};
