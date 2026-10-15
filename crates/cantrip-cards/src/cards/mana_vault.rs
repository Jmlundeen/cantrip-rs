use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

fn add_ccc(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::C, 3);
    Ok(())
}

/// "At the beginning of your draw step, if this artifact is tapped" (CR 603.4): checked as the step begins and again as
/// the ability resolves.
fn your_draw_step(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::Draw } if q.state().turn.active == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn is_tapped(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).tapped
}

/// You may pay {4}. If you do, untap this artifact. Nobody untaps what has left the battlefield.
fn pay_to_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let vault = item.object;
    if !targets::permanent(g, vault) {
        return Ok(());
    }
    if g.unless_pays(item.controller, vault, vault, mana(4, 0, 0, 0, 0, 0))? && targets::permanent(g, vault) {
        g.untap(vault);
    }
    Ok(())
}

/// It deals 1 damage to you: dealt by the permanent, and still dealt by one that has left, off the status it last had.
fn burn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, Target::Player(item.controller), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "This artifact doesn't untap during your untap step."
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        mana::tap_for_n(add_ccc, Mana::C, 3, "{T}: Add {C}{C}{C}."),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your upkeep, you may pay {4}. If you do, untap this artifact.",
            listens: EventMask::STEP_BEGAN,
            matches: triggers::your_upkeep,
            effect: pay_to_untap,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.",
            listens: EventMask::STEP_BEGAN,
            matches: your_draw_step,
            condition: Some(is_tapped),
            effect: burn,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::MANA_VAULT
};
