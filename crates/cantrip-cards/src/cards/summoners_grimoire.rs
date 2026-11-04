use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::job_select::JOB_SELECT;
use cantrip_mechanics::statics::{also_a, grant_delayed_ability};
use cantrip_mechanics::tokens::HERO_1_1_COLORLESS;
use cantrip_mechanics::{choose, hand, lands, select, triggers};
use cantrip_oracle as oracle;

/// "You may put a creature card from your hand onto the battlefield. If that card is an enchantment card, it enters tapped and
/// attacking." (CR 608.2d, 508.4): the card is chosen first and judged as it sits in the hand (CR 614.1c); an enchantment card
/// enters attacking what its controller chooses, or just tapped when there is nothing to attack.
fn put_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let in_hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    let rule = select::creature_cards(select::up_to(1));
    let Some(card) = choose::select(g, item.controller, item.object, Purpose::PutOntoBattlefield, in_hand, rule)?.first().copied() else { return Ok(()) };
    if g.chars(card).is(CardType::Enchantment) {
        hand::put_tapped_and_attacking(g, card, item.controller, item.object)?;
    } else {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[JOB_SELECT, equip(mana("{3}"), "Abraxas \u{2014} Equip {3}")],
    delayed: &[triggered(
        triggers::THIS_ATTACKS,
        put_creature,
        "Whenever this creature attacks, you may put a creature card from your hand onto the battlefield. If that card is an enchantment card, it enters tapped and attacking.",
    )],
    statics: &[
        static_ability(Layer::Type, attached_object, also_a::<{ lands::sub(Subtype::Shaman) }>, "Equipped creature is a Shaman in addition to its other types."),
        static_ability(
            Layer::Ability,
            attached_object,
            grant_delayed_ability,
            "Equipped creature has \"Whenever this creature attacks, you may put a creature card from your hand onto the battlefield. If that card is an enchantment card, it enters tapped and attacking.\"",
        ),
    ],
    ..oracle::SUMMONERS_GRIMOIRE
};

pub static EXTRA: &[&CardDef] = &[&HERO_1_1_COLORLESS];
