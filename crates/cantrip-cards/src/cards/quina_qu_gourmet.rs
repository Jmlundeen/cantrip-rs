use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::replacements::tokens_under_your_control;
use cantrip_oracle as oracle;

/// "... those tokens plus a 1/1 green Frog creature token are created instead" (CR 614.1a): one Frog more in the same
/// event, however many kinds of token it makes (`TokenCreation::plus`). It is created as they are, so what the effect
/// says of them - tapped, attacking, exiled later - says it of the Frog too, but it has none of their abilities
/// (ruling 2025-06-06). The Frog is what Quina's text says now (CR 612.4, 612.2a): Artificial Evolution's Goblin.
fn plus_a_frog(q: &dyn Query, me: &Asker, p: &mut Proposal) {
    if let Proposal::TokenCreation(t) = p {
        let frog = q.card_id(&FROG);
        t.plus = Some((frog, q.token_copy(frog, me.source)));
    }
}

/// "Sacrifice a Frog" (CR 701.21a, 205.3m): the creature type read through Quina's words (CR 612). Only a permanent its
/// controller controls can be sacrificed, which the cost sees to.
fn frog(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Frog))
}

/// The 1/1 green Frog creature token Quina adds.
static FROG: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    ..token!(Frog)
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        Cost::mana(mana("{2}")).and(sacrifice(1, frog)),
        effects::plus_one_counter_on_this,
        "{2}, Sacrifice a Frog: Put a +1/+1 counter on Quina.",
    )],
    // "If one or more tokens would be created under your control": whose they would be, not who makes them (ruling
    // 2025-06-06).
    replacements: &[replacement(
        ReplacementKind::TokenCreation,
        tokens_under_your_control,
        plus_a_frog,
        "creates those tokens plus a 1/1 green Frog creature token instead",
    )],
    ..oracle::QUINA_QU_GOURMET
};

pub static EXTRA: &[&CardDef] = &[&FROG];
