use cantrip_api::*;
use cantrip_mechanics::{creatures_you_control, haste_until_end_of_turn, pump, tokens, triggers};
use cantrip_oracle as oracle;

/// The 2/2 colorless Wizard Soldier creature token named Cadet (CR 111.4).
pub static CADET: CardDef = CardDef {
    name: "Cadet",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Wizard, Subtype::Soldier]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

/// "Whenever you activate a loyalty ability" (CR 606.2, 603.2): any permanent's, not only this one's, by the controller of
/// this permanent. The activation is the event that triggers it (CR 602.2), so the trigger goes on the stack above the
/// ability and its token is on the battlefield before that ability resolves (the ruling).
fn you_activate_a_loyalty_ability(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let (_, player, def) = triggers::activation(q, event)?;
    if def.is_loyalty_ability() && player == q.controller(source) { Fired::PLAIN } else { None }
}

/// "Create a 2/2 colorless Wizard Soldier creature token named Cadet."
fn make_cadet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &CADET, 1)?;
    Ok(())
}

/// +1: "Creatures you control get +1/+0 and gain haste until end of turn": the creatures there are as it resolves
/// (CR 611.2c), the Cadet this activation made among them.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for id in g.permanents_of(item.controller) {
        if g.chars(id).is(CardType::Creature) {
            pump(g, item.object, id, 1, 0);
            haste_until_end_of_turn(g, item.object, id);
        }
    }
    Ok(())
}

/// -2: "Discard your hand, then draw a card for each creature you control": the creatures are counted after the discard.
fn minus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.discard_hand(item.controller);
    let creatures = creatures_you_control(g, item.controller);
    g.draw(item.controller, creatures as u32);
    Ok(())
}

/// -3: "Ajani deals 4 damage to each creature except for tokens you control": one damage event for every other creature,
/// dealt at once (CR 120.2, 608.2c). The tokens are judged as it resolves, the Cadet this activation made among them.
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && !(g.obj(id).token && g.controller(id) == item.controller))
        .map(|id| (item.object, Target::Object(id), 4))
        .collect();
    g.damage_together(&hits);
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&CADET];

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask::ABILITY_ACTIVATED, you_activate_a_loyalty_ability),
            make_cadet,
            "Whenever you activate a loyalty ability, create a 2/2 colorless Wizard Soldier creature token named Cadet.",
        ),
        ability(loyalty(1), plus_one, "+1: Creatures you control get +1/+0 and gain haste until end of turn."),
        ability(loyalty(-2), minus_two, "\u{2212}2: Discard your hand, then draw a card for each creature you control."),
        ability(loyalty(-3), minus_three, "\u{2212}3: Ajani deals 4 damage to each creature except for tokens you control."),
    ],
    ..oracle::AJANI_UNRELENTING
};
