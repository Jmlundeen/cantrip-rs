use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

use tokens::RAT_CANT_BLOCK as RAT;

/// "Create X 1/1 black Rat creature tokens with 'This token can't block.' Creatures you control gain haste until end of
/// turn." The tokens come first, as one instruction (CR 101.4), so they are among the creatures you control as the
/// spell resolves; the creatures that gain haste are those there then, and ones you begin to control later in the turn
/// do not (CR 611.2c). X is the value announced (0 makes no tokens, yet the haste still happens).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &RAT, item.x as u32)?;
    g.end_sentence();
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    instruction(g, |g| {
        for creature in creatures {
            cantrip_mechanics::haste_until_end_of_turn(g, item.object, creature);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SONG_OF_TOTENTANZ };

pub static EXTRA: &[&CardDef] = &[&RAT];
