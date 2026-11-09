use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::{self, plus_pt};
use cantrip_mechanics::{library, triggers};
use cantrip_oracle as oracle;

/// "Look at the top five cards of your library. You may put a Klingon or Warrior creature card from among them onto
/// the battlefield. Put the rest on the bottom of your library in a random order." (CR 608.2c, 401.4, 609.3): the
/// five cards are looked at (a smaller library shows what it has), up to one Klingon or Warrior creature among them
/// may enter, and the rest still in the library go to the bottom in a random order. The types read through the
/// granting Sword's words, which the ability keeps as it was put on the stack (`StackItem::words`, CR 113.7a,
/// 613.1f): the ability resolves whether or not the Sword is still attached or on the battlefield (CR 113.7a).
fn look_put_klingon_or_warrior(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = item.object;
    let klingon_or_warrior = move |q: &dyn Query, c: ObjectId| {
        q.chars(c).is(CardType::Creature)
            && (q.chars(c).has_subtype(q.words(creature).subtype(Subtype::Klingon))
                || q.chars(c).has_subtype(q.words(creature).subtype(Subtype::Warrior)))
    };
    library::look_at_top_onto_battlefield(g, item.controller, creature, 5, 1, "Klingon or Warrior", klingon_or_warrior, None, false, library::Rest::BottomRandom)?;
    Ok(())
}

const ATTACK_TEXT: &str = "Whenever this creature attacks, look at the top five cards of your library. You may put a Klingon or Warrior creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.";

pub static CARD: CardDef = CardDef {
    delayed: &[triggered(triggers::THIS_ATTACKS, look_put_klingon_or_warrior, ATTACK_TEXT)],
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
        static_ability(
            Layer::Ability,
            attached_object,
            statics::grant_delayed_ability,
            "Equipped creature has \"Whenever this creature attacks, look at the top five cards of your library. You may put a Klingon or Warrior creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.\"",
        ),
    ],
    ..oracle::THE_SWORD_OF_KAHLESS
};
