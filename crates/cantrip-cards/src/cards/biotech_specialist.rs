use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, create a Lander token."
fn lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_lander(g, item.controller)
}

/// "Whenever you sacrifice an artifact" (CR 701.21a): an artifact of yours that a sacrifice moved from the battlefield,
/// read as it was when it left, so a Lander sacrificed as a cost counts and so does a Treasure.
fn you_sacrifice_an_artifact(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let sacrificed = triggers::sacrificed_by(q, me.controller, event)?;
    if q.chars(sacrificed).is(CardType::Artifact) { Fired::PLAIN } else { None }
}

/// "This creature deals 2 damage to target opponent": dealt by the creature, off its last-known information if it has
/// left the battlefield (CR 608.2h).
fn two_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, lander, "When this creature enters, create a Lander token."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, you_sacrifice_an_artifact),
            two_damage,
            "Whenever you sacrifice an artifact, this creature deals 2 damage to target opponent.",
        )
        .on_sacrifice()
        .targets(&[targets::OPPONENT]),
    ],
    ..oracle::BIOTECH_SPECIALIST
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
