use cantrip_api::*;
use cantrip_mechanics::shuffle_hands_and_graveyards_in_and_draw_seven;
use cantrip_mechanics::triggers::THIS_ENTERS_IF_CAST;
use cantrip_oracle as oracle;

/// "Shuffle your hand and graveyard into your library, then draw seven cards" (CR 701.24, 121.1): the hand and graveyard move
/// together, only the controller's own, then the library is shuffled and seven are drawn. The enchantment is on the battlefield,
/// so it is in neither.
fn shuffle_in_and_draw_seven(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    shuffle_hands_and_graveyards_in_and_draw_seven(g, &[item.controller]);
    Ok(())
}

/// "The first spell each player casts during each of their turns may be cast without paying its mana cost" (CR 118.9, 601.2b): an
/// alternative cost any player may use for the first spell they cast in a turn of their own, whether from hand or elsewhere. The
/// history holds only the spells cast already, so none of theirs this turn means this is their first; an opponent's spell on my
/// turn is not "during each of their turns" and pays its cost.
fn first_spell_in_own_turn(q: &dyn Query, _: &Asker, _: ObjectId, player: PlayerId) -> bool {
    q.state().turn.active == player && q.history().casts_by(player).next().is_none()
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS_IF_CAST,
        shuffle_in_and_draw_seven,
        "When this enchantment enters, if you cast it, shuffle your hand and graveyard into your library, then draw seven cards.",
    )],
    spell_alternatives: &[SpellAlternative { text: "Without paying its mana cost", applies: first_spell_in_own_turn, cost: InsteadOfManaCost::WithoutPaying, flash: false, kind: WayKind::Other }],
    ..oracle::WEFTWALKING
};
