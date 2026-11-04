use cantrip_api::*;
use cantrip_mechanics::{choose, haste_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Choose odd or even. Destroy each creature with mana value of the chosen quality. Then gain control of all creatures until
/// end of turn. Untap them. They gain haste until end of turn. (Zero is even.)" The choice is made as the spell resolves
/// (CR 608.2d) and the creatures are destroyed together (CR 701.8). "Then" starts a new sentence: the creatures taken are those
/// still on the battlefield after the destruction, every player's, the caster's own included (it gains control of those too,
/// and untaps and hastes them). The effects affect just them, not a creature that enters later in the turn (CR 611.2c). Each
/// sentence is judged by the state triggers before the next (CR 603.8, 608.2c).
fn massacre(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.card_id(&CARD);
    let doomed = choose::creatures_of_chosen_quality(g, item, me)?;
    g.destroy_all(&doomed);
    g.end_sentence();
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    instruction(g, |g| {
        for &creature in &creatures {
            g.gain_control(item.object, creature, item.controller, Duration::EndOfTurn);
        }
    });
    g.end_sentence();
    for &creature in &creatures {
        g.untap(creature);
    }
    g.end_sentence();
    instruction(g, |g| {
        for &creature in &creatures {
            haste_until_end_of_turn(g, item.object, creature);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { labels: choose::ODD_OR_EVEN, spell: spell(&[], massacre), ..oracle::MUTINOUS_MASSACRE };
