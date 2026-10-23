use cantrip_api::*;
use cantrip_mechanics::select::{self, exactly};
use cantrip_mechanics::{choose, destroy_target, effects, targets};
use cantrip_oracle as oracle;

/// Mode 1: "Target player mills three cards, then you return a land card from your graveyard to your hand." The mill (CR 701.17)
/// is that player's, made only if the target is still legal (CR 608.2b); the return is yours and does not use it, so it is made
/// either way, from the graveyard as the mill left it - a land card just milled into your own is one to return (ruling
/// 2021-04-16). A land card is returned if there is one, so nothing is asked of a graveyard without one.
fn mill_then_return_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Player(player) = item.target(0)
    {
        g.mill(player, 3);
    }
    g.end_sentence();
    let lands: Vec<ObjectId> = g.state().zone(Zone::graveyard(item.controller)).iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    let chosen = choose::select(g, item.controller, item.object, Purpose::PutIntoHand, lands, select::land_cards(exactly(1)))?;
    g.return_to_hand(&chosen);
    Ok(())
}

/// Mode 2: "Destroy target noncreature, nonland permanent with mana value 2 or less."
fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(mill_then_return_a_land, "Target player mills three cards, then you return a land card from your graveyard to your hand.")
        .targets(&[targets::PLAYER]),
    mode(destroy_it, "Destroy target noncreature, nonland permanent with mana value 2 or less.")
        .targets(&[target(targets::noncreature_nonland_permanent_with_mana_value_at_most::<2>)]),
    mode(effects::target_creature_gets::<-3, -1>, "Target creature gets -3/-1 until end of turn.")
        .targets(&[targets::CREATURE]),
    mode(|g, item| effects::target_player_loses_life_you_gain_life(g, item, 2), "Target opponent loses 2 life and you gain 2 life.")
        .targets(&[targets::OPPONENT]),
])
    .choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::WITHERBLOOM_COMMAND };
