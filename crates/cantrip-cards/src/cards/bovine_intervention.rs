use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Destroy target artifact or creature. Its controller creates a 2/2 white Ox creature token." The controller is read
/// before the destroy (CR 608.2h), so it is the player who controlled the target as the spell resolved, and the token is
/// created whether or not it was destroyed (indestructible, regeneration; ruling 2024-04-12). A target that is illegal on
/// resolution makes the spell not resolve at all (CR 608.2b), so no token is created then.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(target);
    g.destroy(target);
    tokens::create(g, controller, &OX, 1)?;
    Ok(())
}

/// The 2/2 white Ox creature token.
pub static OX: CardDef = CardDef {
    name: "Ox",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Ox]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

pub static EXTRA: &[&CardDef] = &[&OX];

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ARTIFACT_OR_CREATURE], effect), ..oracle::BOVINE_INTERVENTION };
