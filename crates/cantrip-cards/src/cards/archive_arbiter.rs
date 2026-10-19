use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "A noncreature, nonland permanent" (CR 115.1): a permanent that is neither, so no creature that is also a land.
fn noncreature_nonland(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && !q.chars(id).is(CardType::Creature) && !q.chars(id).is(CardType::Land))
}

/// Mode 0: destroy target noncreature, nonland permanent.
fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_it, "Destroy target noncreature, nonland permanent.").targets(&[target(noncreature_nonland)]),
    mode(|g, item| effects::gain_life(g, item, 4), "You gain 4 life."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(THIS_ENTERS, &MODAL, "When this creature enters, choose one — Destroy target noncreature, nonland permanent; or you gain 4 life.")],
    ..oracle::ARCHIVE_ARBITER
};
