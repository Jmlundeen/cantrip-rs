use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

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
        mana_ability(TAP, add_n(Mana::C, 3), "{T}: Add {C}{C}{C}."),
        triggered(triggers::YOUR_UPKEEP, pay_to_untap, "At the beginning of your upkeep, you may pay {4}. If you do, untap this artifact."),
        triggered(when(EventMask::STEP_BEGAN, your_draw_step), burn, "At the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.")
            .condition(is_tapped),
    ],
    ..oracle::MANA_VAULT
};
