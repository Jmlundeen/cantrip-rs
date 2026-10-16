use cantrip_api::*;
use cantrip_mechanics::evoke;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// The mana that was spent to cast the permanent (CR 106.1b, 202.2), counted over the whole payment, so mana spent on
/// the evoke cost counts and one more of the color changes nothing. A permanent that was never cast spent none
/// (CR 400.7). Read from the object the ability came from, so it still answers when that left the battlefield.
fn two_spent(q: &dyn Query, source: ObjectId, mana: Mana) -> bool {
    q.obj(source).spent.0[mana as usize] >= 2
}

fn red_red_spent(q: &dyn Query, asker: &Asker) -> bool {
    two_spent(q, asker.source, Mana::R)
}

fn green_green_spent(q: &dyn Query, asker: &Asker) -> bool {
    two_spent(q, asker.source, Mana::G)
}

fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

/// Search your library for a land card, reveal it, put it into your hand, then shuffle. You gain 2 life.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    search_to_hand(g, player, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Land))?;
    g.gain_life(player, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        evoke::sacrifice_trigger("When Vibrance enters, if its evoke cost was paid, its controller sacrifices it."),
        triggered(THIS_ENTERS, damage, "When this creature enters, if {R}{R} was spent to cast it, this creature deals 3 damage to any target.")
            .targets(&[targets::ANY])
            .condition(red_red_spent),
        triggered(THIS_ENTERS, search, "When this creature enters, if {G}{G} was spent to cast it, search your library for a land card, reveal it, put it into your hand, then shuffle. You gain 2 life.")
            .condition(green_green_spent),
    ],
    ways: &[evoke::way(Cost::mana(ManaCost {
            generic: 0,
            symbols: [0; 6],
            hybrid: [Some((Mana::R, Mana::G)), Some((Mana::R, Mana::G)), None, None, None, None],
            phyrexian: [0; 5],
            x: 0,
        }), "Evoke {R/G}{R/G}")],
    ..oracle::VIBRANCE
};
