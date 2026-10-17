use cantrip_api::*;
use cantrip_mechanics::{each_opponent_sacrifices, targets};
use cantrip_oracle as oracle;

fn artifact(q: &dyn Query, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Artifact)
}

fn enchantment(q: &dyn Query, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Enchantment)
}

/// "Each opponent sacrifices an artifact of their choice" (CR 701.21a, 101.4): each opponent in turn order chooses among the
/// artifacts they control, tokens included; one with a single artifact has no choice, and one with none sacrifices nothing.
fn sacrifice_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, artifact)
}

/// "Each opponent sacrifices an enchantment of their choice."
fn sacrifice_enchantment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, enchantment)
}

/// "Each opponent sacrifices a creature with flying of their choice": flying as the layers give it as the spell resolves.
fn sacrifice_flier(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, targets::has_flying_creature)
}

static MODAL: ModalDef = modes(&[
    mode(sacrifice_artifact, "Each opponent sacrifices an artifact of their choice."),
    mode(sacrifice_enchantment, "Each opponent sacrifices an enchantment of their choice."),
    mode(sacrifice_flier, "Each opponent sacrifices a creature with flying of their choice."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::PICK_YOUR_POISON };
