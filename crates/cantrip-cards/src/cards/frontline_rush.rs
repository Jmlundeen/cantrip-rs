use cantrip_api::*;
use cantrip_mechanics::{creatures_you_control, pump, targets, tokens};
use cantrip_oracle as oracle;

/// The 1/1 red Goblin creature token.
pub static GOBLIN: CardDef = CardDef {
    name: "Goblin",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Goblin]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// "Create two 1/1 red Goblin creature tokens."
fn goblins(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &GOBLIN, 2)?;
    Ok(())
}

/// "Target creature gets +X/+X until end of turn, where X is the number of creatures you control." X is worked out once,
/// as this resolves (official ruling); with the target illegal nothing happens (CR 608.2b).
fn rush(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let x = creatures_you_control(g, item.controller) as i16;
        pump(g, item.object, creature, x, x);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(goblins, "Create two 1/1 red Goblin creature tokens."),
    mode(rush, "Target creature gets +X/+X until end of turn, where X is the number of creatures you control.")
        .targets(&[targets::CREATURE]),
])
.choose(1, 1);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::FRONTLINE_RUSH };

pub static EXTRA: &[&CardDef] = &[&GOBLIN];
