use cantrip_api::*;
use cantrip_mechanics::{copy, effects, targets};
use cantrip_mechanics::triggers::THIS_ATTACKS_SUBJECT;
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, target creature defending player controls gets -3/-0 until end of turn, then create a token
/// that's a copy of this creature." (CR 603.3, 608.2c): the pump is the first sentence and the copy the second, in that order.
/// The target must still be the defending player's creature as the ability resolves (CR 608.2b) or it is not pumped, and the
/// token is made either way; the copy uses this creature's copiable values as it is now, so it is a 3/3 Germ with flying and
/// does not enter attacking (CR 508.4, 707.2). The target filter reads the attacking creature from the trigger's subject,
/// so the ability is `THIS_ATTACKS_SUBJECT` rather than a plain attack trigger (CR 508.5).
fn shrink_then_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_creature_gets::<-3, 0>(g, item)?;
    g.end_sentence();
    copy::create_token_copy_of_it(g, item, |_, values| values)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS_SUBJECT,
        shrink_then_copy,
        "Whenever this creature attacks, target creature defending player controls gets -3/-0 until end of turn, then create a token that's a copy of this creature.",
    )
    .targets(&[targets::CREATURE_DEFENDING_PLAYER_CONTROLS])],
    ..oracle::GIANT_SPACE_AMOEBA
};
