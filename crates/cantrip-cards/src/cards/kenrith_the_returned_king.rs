use cantrip_api::*;
use cantrip_mechanics::{effects, haste_until_end_of_turn, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "{R}: All creatures gain trample and haste until end of turn." Only the creatures on the battlefield as it resolves
/// gain them; ones that enter later do not (CR 611.2c).
fn all_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .collect();
    instruction(g, |g| {
        for creature in creatures {
            trample_until_end_of_turn(g, item.object, creature);
            haste_until_end_of_turn(g, item.object, creature);
        }
    });
    Ok(())
}

/// "{1}{G}: Put a +1/+1 counter on target creature."
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_target(g, item)
}

/// "{2}{W}: Target player gains 5 life."
fn gain_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.gain_life(player, 5);
    }
    Ok(())
}

/// "{4}{B}: Put target creature card from a graveyard onto the battlefield under its owner's control."
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield(card, owner, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{R}"), all_gain, "{R}: All creatures gain trample and haste until end of turn."),
        ability(cost("{1}{G}"), counter, "{1}{G}: Put a +1/+1 counter on target creature.").targets(&[targets::CREATURE]),
        ability(cost("{2}{W}"), gain_five, "{2}{W}: Target player gains 5 life.").targets(&[targets::PLAYER]),
        ability(cost("{3}{U}"), |g, item| Ok(effects::target_player_draws(g, item, 0, 1)), "{3}{U}: Target player draws a card.").targets(&[targets::PLAYER]),
        ability(cost("{4}{B}"), reanimate, "{4}{B}: Put target creature card from a graveyard onto the battlefield under its owner's control.")
            .targets(&[targets::CREATURE_CARD_IN_A_GRAVEYARD]),
    ],
    ..oracle::KENRITH_THE_RETURNED_KING
};
