use cantrip_api::*;
use cantrip_mechanics::{flurry, tokens};
use cantrip_oracle as oracle;

/// The 1/1 white Human Soldier creature token.
static HUMAN_SOLDIER: CardDef = CardDef {
    name: "Human Soldier",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Human, Subtype::Soldier]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// Mode 0: "Create two 1/1 white Human Soldier creature tokens", together (CR 101.4).
fn make_tokens(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &HUMAN_SOLDIER, 2)?;
    Ok(())
}

/// Mode 1: "Put a +1/+1 counter on each creature you control", the creatures read as the ability resolves, which is
/// before the spell that triggered it (CR 603.3), so that spell's creature gets none.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        g.add_counters(id, CounterKind::PlusOne, 1);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(make_tokens, "Create two 1/1 white Human Soldier creature tokens."),
    mode(counters, "Put a +1/+1 counter on each creature you control."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[flurry::modal_trigger(&MODAL, "Whenever you cast your second spell each turn, choose one \u{2014} Create two 1/1 white Human Soldier creature tokens.; or Put a +1/+1 counter on each creature you control.")],
    ..oracle::COSMOGRAND_ZENITH
};

pub static EXTRA: &[&CardDef] = &[&HUMAN_SOLDIER];
