use cantrip_api::*;
use cantrip_mechanics::{harness, mana, targets};
use cantrip_oracle as oracle;

/// "Up to one other target nonland permanent you control."
fn other_nonland_permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if id != asker.source
            && is_live(q, id)
            && q.is_on_battlefield(id)
            && q.controller(id) == asker.controller
            && !q.chars(id).is(CardType::Land))
}

/// Exile it, then return that card to the battlefield under its owner's control: a new object (CR 400.7).
fn blink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(permanent).owner;
    let exiled = g.exile(permanent);
    if g.obj(exiled).zone.is(ZoneKind::Exile) && !g.obj(exiled).token {
        g.put_onto_battlefield(exiled, owner, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_w, Mana::W, "{T}: Add {W}."),
        harness::ability(
            Cost { mana: mana(5, 1, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            "{5}{W}, {T}: Harness The Mind Stone.",
        ),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of your end step, exile up to one other target nonland permanent you control, then return that card to the battlefield under its owner's control.",
        listens: EventMask::STEP_BEGAN,
        matches: cantrip_mechanics::triggers::your_end_step,
        targets: &[TargetSpec { min: 0, max: 1, filter: other_nonland_permanent_you_control }],
        effect: blink,
        ..TriggeredDef::BLANK
    })],
    statics: &[harness::infinity(
        "\u{221e} \u{2014} At the beginning of your end step, exile up to one other target nonland permanent you control, then return that card to the battlefield under its owner's control.",
    )],
    ..oracle::THE_MIND_STONE
};
