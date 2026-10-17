use cantrip_api::*;
use cantrip_mechanics::{evoke, spent};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

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
            .condition(spent::two::<{ Mana::R as u8 }>),
        triggered(THIS_ENTERS, search, "When this creature enters, if {G}{G} was spent to cast it, search your library for a land card, reveal it, put it into your hand, then shuffle. You gain 2 life.")
            .condition(spent::two::<{ Mana::G as u8 }>),
    ],
    ways: &[evoke::way(Cost::mana(mana("{R/G}{R/G}")), "Evoke {R/G}{R/G}")],
    ..oracle::VIBRANCE
};
