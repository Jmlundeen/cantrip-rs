use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_DIES};
use cantrip_oracle as oracle;

/// The Adventure (CR 715): Dread Whispers, "You draw a card and you lose 1 life."
pub static DREAD_WHISPERS: CardDef = CardDef { spell: spell(&[], effects::draw_a_card_and_lose_1_life), ..oracle::DREAD_WHISPERS };

/// "When this creature dies, you may cast it from your graveyard as an Adventure until the end of your next turn": a
/// permission (CR 601.3) for the card it became in the graveyard (CR 400.7), while it stays there, to cast it as Dread
/// Whispers only (CR 715.3), paying its costs and at a sorcery's timing (ruling 2023-09-01). "Your graveyard": a Dreadknight
/// someone else owned went to its owner's (CR 400.3), which is not the ability's controller's, and nothing is granted.
fn may_cast_as_adventure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    if g.obj(card).zone == Zone::graveyard(item.controller) {
        g.allow_play(card, item.controller, PlayGrant { adventure_only: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_DIES,
        may_cast_as_adventure,
        "When this creature dies, you may cast it from your graveyard as an Adventure until the end of your next turn.",
    )],
    adventure: Some(&DREAD_WHISPERS),
    ..oracle::MOSSWOOD_DREADKNIGHT
};

pub static EXTRA: &[&CardDef] = &[&DREAD_WHISPERS];
