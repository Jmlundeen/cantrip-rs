use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_LEAVES;
use cantrip_mechanics::{damage_target, targets, tokens};
use cantrip_oracle as oracle;

/// "When this token leaves the battlefield, it deals 2 damage to any target." (CR 603.6c, 603.10a): the token is the
/// source, looked back on as it last was on the battlefield.
fn munitions_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

/// A colorless artifact token named Munitions.
pub static MUNITIONS: CardDef = CardDef {
    name: "Munitions",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[triggered(THIS_LEAVES, munitions_damage, "When this token leaves the battlefield, it deals 2 damage to any target.")
        .targets(&[targets::ANY])
        .look_back()],
    text: "When this token leaves the battlefield, it deals 2 damage to any target.",
    ..CardDef::BLANK
};

/// "Whenever a nontoken artifact you control enters" (CR 603.6a): judged as it entered, so an artifact creature counts.
fn nontoken_artifact_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && !q.obj(*new).token
                && q.chars(*new).is(CardType::Artifact) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn make_munitions(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &MUNITIONS, 1)?;
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&MUNITIONS];

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, nontoken_artifact_you_control_enters),
        make_munitions,
        "Whenever a nontoken artifact you control enters, create a colorless artifact token named Munitions with \"When this token leaves the battlefield, it deals 2 damage to any target.\"",
    )],
    ..oracle::WEAPONS_MANUFACTURING
};
