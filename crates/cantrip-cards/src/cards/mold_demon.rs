use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// A Swamp, read through Mold Demon's words (CR 612): "two Swamps".
fn swamp(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Swamp))
}

/// "When this creature enters, sacrifice it unless you sacrifice two Swamps." (CR 608.2c, 701.21a, 118.12): a payer who
/// can't sacrifice two Swamps is not asked (CR 118.3).
fn sacrifice_unless_sacrifice_two_swamps(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.may_pay(item.controller, item.object, item.object, sacrifice(2, swamp), &PAY_QUESTION)? {
        g.sacrifice(item.controller, &[item.object]);
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new(
    "Pay",
    "sacrifice two Swamps, or sacrifice {subject}?",
    "",
    "Sacrifice two Swamps",
    "Sacrifice {subject}",
)
.no_default();

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            sacrifice_unless_sacrifice_two_swamps,
            "When this creature enters, sacrifice it unless you sacrifice two Swamps.",
        ),
    ],
    ..oracle::MOLD_DEMON
};
