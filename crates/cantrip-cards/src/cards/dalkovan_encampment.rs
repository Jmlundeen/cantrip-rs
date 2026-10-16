use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_mechanics::{delayed, mobilize, triggers};
use cantrip_oracle as oracle;

/// "{2}{W}, {T}: Whenever you attack this turn, ...": sets up the delayed triggered ability of "whenever you attack this
/// turn" (CR 603.7b), controlled by whoever activated this as it resolved (CR 603.7d). It outlives the land and triggers each
/// time its controller declares attackers this turn (CR 508.1m), however many times the ability was activated, and not
/// at all when no creature is declared as an attacker.
fn attack_this_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

/// "Create two 1/1 red Warrior creature tokens that are tapped and attacking" (CR 508.4): their controller chooses what
/// each one attacks. "Sacrifice them at the beginning of the next end step" (CR 603.7c): the sacrifice is the card's second
/// delayed ability, about exactly the tokens made. The Warriors are made from this card's text, so a text change reaches
/// their color and creature type (CR 612.2a).
fn warriors(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let warrior = g.card_id(&mobilize::WARRIOR);
    let created = create_attacking_of_your_choice(g, item.controller, item.object, warrior, 2)?;
    delayed::at_next_end_step_about(g, item, 1, &created);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Swamp) }, { sub(Subtype::Mountain) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(cost("{2}{W}, {T}"), attack_this_turn, "{2}{W}, {T}: Whenever you attack this turn, create two 1/1 red Warrior creature tokens that are tapped and attacking. Sacrifice them at the beginning of the next end step."),
    ],
    delayed: &[
        triggered(triggers::YOU_ATTACK, warriors, "Whenever you attack this turn, create two 1/1 red Warrior creature tokens that are tapped and attacking. Sacrifice them at the beginning of the next end step."),
        delayed::SACRIFICE_CREATED,
    ],
    ..oracle::DALKOVAN_ENCAMPMENT
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
