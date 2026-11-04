use cantrip_api::*;
use cantrip_mechanics::{effects, prevention, targets};
use cantrip_oracle as oracle;

/// "Creatures you control gain hexproof and indestructible until end of turn. You gain hexproof until end of turn. Prevent
/// all damage that would be dealt to you this turn. Exile The Needs of the Many." Each part is one effect made as the spell
/// resolves (CR 608.2): the creatures are those its controller controls then, the hexproof is the player's (CR 702.11c),
/// the prevention a rule about all damage to that player (CR 615.1), and the spell exiles itself rather than going to the
/// graveyard (CR 608.2m).
fn needs_of_the_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = item.controller;
    let creatures: Vec<ObjectId> = g
        .permanents_of(me)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .collect();
    // One instruction, one continuous effect applying to each of those creatures (CR 611.2c), not one per creature.
    instruction(g, |g| {
        for creature in creatures {
            effects::gain_hexproof_and_indestructible_until_end_of_turn(g, item.object, creature);
        }
    });
    g.end_sentence();
    g.add_effect(item.object, item.object, EffectKind::PlayerHexproof, Duration::EndOfTurn);
    g.end_sentence();
    prevention::this_turn(g, item, Target::Player(me), 0, Shield::All);
    g.end_sentence();
    g.exile(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], needs_of_the_many),
    // As an additional cost to cast this spell, sacrifice a creature (CR 601.2b, 701.21a).
    additional_choice: &[ExtraCost { text: "Sacrifice a creature", cost: Cost::sacrifice(1, targets::filters::creature) }],
    granted_preventions: &[prevention::ANY_DAMAGE.text("Prevent all damage that would be dealt to you this turn.")],
    ..oracle::THE_NEEDS_OF_THE_MANY
};
