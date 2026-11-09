use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Create two 1/1 blue Faerie creature tokens with flying. When you do, tap target creature an opponent controls."
/// The reflexive ability (CR 603.12) triggers only if the tokens were created, and its target is chosen as it is put on
/// the stack, so it is not chosen when the spell is cast (ruling); with no creature an opponent controls it is not put on
/// the stack.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let created = tokens::create(g, item.controller, &tokens::FAERIE, 2)?;
    if !created.is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // "Tap target creature an opponent controls" (CR 701.26a): a target that has become illegal is left alone (CR 608.2b).
    delayed: &[delayed_trigger(effects::tap_targets,"When you do, tap target creature an opponent controls.").targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::FAEBLOOM_TRICK
};

pub static EXTRA: &[&CardDef] = &[&tokens::FAERIE];
