use cantrip_api::*;
use cantrip_mechanics::tokens::{self, CONSTRUCT};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::scry;
use cantrip_oracle as oracle;

/// "When this artifact enters, scry 2" (CR 701.22).
fn scry_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 2)
}

/// "Whenever another artifact you control with mana value 3 or greater enters" (CR 603.6a): an artifact other than this one,
/// under its controller's control as it entered, whose mana value (CR 202.3, `Query::mana_value`: a token or a copy as the
/// rules read it) is 3 or more.
fn another_big_artifact_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != me.source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).is(CardType::Artifact)
                && q.mana_value(*new) >= 3 =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Create a 0/0 colorless Construct artifact creature token with 'This token gets +1/+1 for each artifact you control.'"
fn make_construct(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &CONSTRUCT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, scry_two, "When this artifact enters, scry 2."),
        triggered(when_asked(EventMask::ZONE_CHANGE, another_big_artifact_you_control_enters), make_construct, "Whenever another artifact you control with mana value 3 or greater enters, create a 0/0 colorless Construct artifact creature token with \"This token gets +1/+1 for each artifact you control.\""),
    ],
    ..oracle::SIMULACRUM_SYNTHESIZER
};

pub static EXTRA: &[&CardDef] = &[&CONSTRUCT];
