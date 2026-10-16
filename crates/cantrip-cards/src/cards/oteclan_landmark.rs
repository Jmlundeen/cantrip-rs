use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_ENTERS};
use cantrip_mechanics::{craft, scry, targets};
use cantrip_oracle as oracle;

/// "When this artifact enters, scry 2." (CR 701.22)
fn scry_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 2)
}

/// "Target attacking creature without flying": attacking now (CR 506.3), and judged again as the ability resolves (CR
/// 608.2b), so one that has gained flying since is no longer a legal target.
fn attacking_without_flying(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.is_attacking(id) && !q.chars(id).has(Keyword::Flying))
}

/// "... gains flying until end of turn."
fn gains_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(THIS_ATTACKS, gains_flying, "Whenever this creature attacks, target attacking creature without flying gains flying until end of turn.")
        .targets(&[target(attacking_without_flying)])],
    ..oracle::OTECLAN_LEVITATOR
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, scry_two, "When this artifact enters, scry 2."),
        craft::ability(mana("{2}{W}"), &craft::ARTIFACT, "Craft with artifact {2}{W}"),
    ],
    back: Some(&BACK),
    ..oracle::OTECLAN_LANDMARK
};
