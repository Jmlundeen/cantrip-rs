use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::library;
use cantrip_oracle as oracle;

/// "Equipped creature gets +2/+2 and has" (CR 613.1f): the second half of the line - the equipped creature gets
/// the Sword's attack trigger ('Whenever this creature attacks, look at the top five cards of your library. You may
/// put a Klingon or Warrior creature card from among them onto the battlefield. Put the rest on the bottom of your
/// library in a random order.'), so the trigger is the creature's (its controller looks) and each Sword attached
/// to it gives one.
fn grant_attack_trigger(view: &dyn LayerQuery, _: ObjectId, _creature: ObjectId, chars: &mut Chars) {
    let back = view.cards().id_of(&BACK);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(back, 0));
    }
}

/// "Whenever this creature attacks" (CR 508.1): the creature with the granted ability, declared as an attacker.
fn this_creature_attacks(_q: &dyn Query, creature: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == creature => Fired::PLAIN,
        _ => None,
    }
}

/// "Look at the top five cards of your library. You may put a Klingon or Warrior creature card from among them onto
/// the battlefield. Put the rest on the bottom of your library in a random order." (CR 608.2c, 401.4, 609.3): the
/// five cards are looked at (a smaller library shows what it has), up to one Klingon or Warrior creature among them
/// may enter, and the rest still in the library go to the bottom in a random order. The types read through the
/// granting Sword's words (CR 612).
fn look_put_klingon_or_warrior(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = item.object;
    let front = g.card_id(&CARD);
    let sword = g.state().battlefield().iter().copied().find(|&id| g.obj(id).attached_to == Some(creature) && g.chars(id).card == front);
    let Some(sword) = sword else { return Ok(()) };
    let klingon_or_warrior = move |q: &dyn Query, c: ObjectId| {
        q.chars(c).is(CardType::Creature)
            && (q.chars(c).has_subtype(q.words(sword).subtype(Subtype::Klingon))
                || q.chars(c).has_subtype(q.words(sword).subtype(Subtype::Warrior)))
    };
    library::look_at_top_onto_battlefield(g, item.controller, sword, 5, 1, "Klingon or Warrior", klingon_or_warrior, None, false, library::Rest::BottomRandom)?;
    Ok(())
}

const ATTACK_TEXT: &str = "Whenever this creature attacks, look at the top five cards of your library. You may put a Klingon or Warrior creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.";

pub static BACK: CardDef = CardDef {
    delayed: &[triggered(when(EventMask::ATTACKS, this_creature_attacks), look_put_klingon_or_warrior, ATTACK_TEXT)],
    ..oracle::THE_SWORD_OF_KAHLESS
};

pub static CARD: CardDef = CardDef {
    back: Some(&BACK),
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
        static_ability_for_each(
            Layer::Ability,
            attached_object,
            grant_attack_trigger,
            "Equipped creature has \"Whenever this creature attacks, look at the top five cards of your library. You may put a Klingon or Warrior creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.\"",
        ),
    ],
    ..oracle::THE_SWORD_OF_KAHLESS
};
