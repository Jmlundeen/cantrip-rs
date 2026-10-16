use cantrip_api::*;
use cantrip_mechanics::{harness, targets};
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
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        harness::ability(
            cost("{5}{W}, {T}"),
            "{5}{W}, {T}: Harness The Mind Stone.",
        ),
    ],
    delayed: &[triggered(cantrip_mechanics::triggers::YOUR_END_STEP, blink, "At the beginning of your end step, exile up to one other target nonland permanent you control, then return that card to the battlefield under its owner's control.")
        .targets(&[up_to_one(other_nonland_permanent_you_control)])],
    statics: &[harness::infinity(
        "\u{221e} \u{2014} At the beginning of your end step, exile up to one other target nonland permanent you control, then return that card to the battlefield under its owner's control.",
    )],
    ..oracle::THE_MIND_STONE
};
