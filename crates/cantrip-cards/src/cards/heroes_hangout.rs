use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Date Night - Exile the top two cards of your library. Choose one of them. Until the end of your next turn, you may play
/// that card": only the chosen card, a land under the ordinary land-play rules (CR 305.1), until the cleanup step of
/// the controller's next turn (CR 514.2).
fn date_night(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let grant = PlayGrant { instead: None, lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST };
    library::exile_top_choose_one_may_play(g, item.controller, item.object, 2, grant)?;
    Ok(())
}

/// "Patrol Night - One or two target creatures each get +1/+0 and gain first strike until end of turn."
fn patrol_night(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = targets::legal_objects(g, item);
    instruction(g, |g| {
        for creature in creatures {
            g.add_effect(item.object, creature, EffectKind::Pump(1, 0), Duration::EndOfTurn);
            g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::FirstStrike])), Duration::EndOfTurn);
        }
    });
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(date_night, "Date Night — Exile the top two cards of your library. Choose one of them. Until the end of your next turn, you may play that card."),
    mode(patrol_night, "Patrol Night — One or two target creatures each get +1/+0 and gain first strike until end of turn.")
        .targets(&[targets::CREATURE.between(1, 2)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::HEROES_HANGOUT };
