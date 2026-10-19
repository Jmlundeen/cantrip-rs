use cantrip_api::*;
use cantrip_mechanics::{effects, tokens, triggers};
use cantrip_oracle as oracle;

/// Peer Review, the prepare spell (CR 722): "Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1."
pub static PEER_REVIEW: CardDef = CardDef {
    name: "Peer Review",
    cost: mana("{2}{W/U}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], |g, item| {
        tokens::create(g, item.controller, &tokens::CADET, 1)?;
        effects::surveil(g, item, 1)
    }),
    text: "Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1.",
    ..CardDef::BLANK
};

/// "Creatures you control get +1/+0 until end of turn": the creatures its controller controls as it resolves (CR
/// 611.2c), and no creature that enters later.
fn pump_team(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::pump_creatures_you_control(g, item, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_SCRY_OR_SURVEIL,
        pump_team,
        "Whenever you scry or surveil, creatures you control get +1/+0 until end of turn. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)],
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&PEER_REVIEW),
    ..oracle::PRUDENT_FATESEER
};

pub static EXTRA: &[&CardDef] = &[&PEER_REVIEW, &tokens::CADET];
